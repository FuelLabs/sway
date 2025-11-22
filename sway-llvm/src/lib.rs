//! Experimental LLVM backend for Sway IR using Inkwell.
//!
//! This is the initial scaffolding for translating `sway-ir` modules into LLVM IR. The goal is to
//! grow this incrementally: start with type lowering and function declarations, then add
//! instruction lowering in tiers.

use std::collections::HashMap;

use inkwell::{
    builder::Builder,
    context::Context as LlvmContext,
    module::Module as LlvmModule,
    targets::TargetTriple,
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum},
    AddressSpace,
};
use sway_ir::{
    function::Function,
    irtype::{Type, TypeContent},
    module::Module,
};
use thiserror::Error;

/// Options that influence LLVM module emission.
#[derive(Debug, Clone, Default)]
pub struct BackendOptions {
    pub target_triple: Option<String>,
    pub data_layout: Option<String>,
}

#[derive(Debug, Error)]
pub enum LlvmError {
    #[error("unsupported type: {0}")]
    UnsupportedType(String),
    #[error("unsupported instruction lowering: {0}")]
    UnsupportedInstruction(&'static str),
    #[error("lowering error: {0}")]
    Lowering(String),
}

pub type Result<T> = std::result::Result<T, LlvmError>;

/// Lower a Sway IR module into LLVM IR text. This is intentionally narrow for now, focusing on
/// type translation and function signatures so that we can layer in instruction lowering next.
pub fn lower_module_to_string<'eng>(
    ir: &sway_ir::Context<'eng>,
    module: Module,
    opts: &BackendOptions,
) -> Result<String> {
    let llvm = LlvmContext::create();
    let mut lowerer = ModuleLowerer::new(&llvm, ir, module, opts.clone())?;
    lowerer.lower_module_decls()?;

    Ok(lowerer.llvm_module.print_to_string().to_string())
}

struct ModuleLowerer<'ctx, 'ir, 'eng> {
    llvm: &'ctx LlvmContext,
    ir: &'ir sway_ir::Context<'eng>,
    ir_module: Module,
    llvm_module: LlvmModule<'ctx>,
    _builder: Builder<'ctx>,
    _opts: BackendOptions,
    type_cache: HashMap<Type, LoweredType<'ctx>>,
    func_map: HashMap<Function, inkwell::values::FunctionValue<'ctx>>,
}

impl<'ctx, 'ir, 'eng> ModuleLowerer<'ctx, 'ir, 'eng> {
    fn new(
        llvm: &'ctx LlvmContext,
        ir: &'ir sway_ir::Context<'eng>,
        ir_module: Module,
        opts: BackendOptions,
    ) -> Result<Self> {
        let llvm_module = llvm.create_module("sway_module");
        if let Some(triple) = &opts.target_triple {
            let target_triple = TargetTriple::create(triple);
            llvm_module.set_triple(&target_triple);
        }

        Ok(Self {
            llvm,
            ir,
            ir_module,
            llvm_module,
            _builder: llvm.create_builder(),
            _opts: opts,
            type_cache: HashMap::new(),
            func_map: HashMap::new(),
        })
    }

    /// For now we only emit function declarations. Bodies and instruction lowering will follow.
    fn lower_module_decls(&mut self) -> Result<()> {
        for func in self.ir_module.function_iter(self.ir) {
            let fn_val = self.declare_function(func)?;
            self.func_map.insert(func, fn_val);
        }
        Ok(())
    }

    fn declare_function(&mut self, func: Function) -> Result<inkwell::values::FunctionValue<'ctx>> {
        let ret_ty = func.get_return_type(self.ir);
        let lowered_ret = self.lower_type(ret_ty)?;

        let mut arg_tys = Vec::with_capacity(func.num_args(self.ir));
        for (_, arg_val) in func.args_iter(self.ir) {
            let arg_ty = arg_val
                .get_type(self.ir)
                .ok_or_else(|| LlvmError::Lowering("function argument missing type".into()))?;
            arg_tys.push(self.lower_basic_metadata_type(arg_ty)?);
        }

        let fn_type = match lowered_ret {
            LoweredType::Void => self.llvm.void_type().fn_type(arg_tys.as_slice(), false),
            LoweredType::Basic(ret) => ret.fn_type(arg_tys.as_slice(), false),
        };

        Ok(self
            .llvm_module
            .add_function(func.get_name(self.ir), fn_type, None))
    }

    fn lower_basic_metadata_type(&mut self, ty: Type) -> Result<BasicMetadataTypeEnum<'ctx>> {
        match self.lower_type(ty)? {
            LoweredType::Void => Err(LlvmError::UnsupportedType(
                "void not valid in argument position".into(),
            )),
            LoweredType::Basic(basic) => Ok(basic.into()),
        }
    }

    fn lower_type(&mut self, ty: Type) -> Result<LoweredType<'ctx>> {
        if let Some(cached) = self.type_cache.get(&ty) {
            return Ok(*cached);
        }

        let lowered = match ty.get_content(self.ir) {
            TypeContent::Unit => LoweredType::Void,
            TypeContent::Bool => LoweredType::Basic(self.llvm.bool_type().into()),
            TypeContent::Uint(bits) => {
                LoweredType::Basic(self.llvm.custom_width_int_type((*bits).into()).into())
            }
            TypeContent::B256 => LoweredType::Basic(self.llvm.custom_width_int_type(256).into()),
            TypeContent::Array(elm, len) => {
                let lowered_elm = self.lower_type(*elm)?.as_basic()?;
                LoweredType::Basic(lowered_elm.array_type(*len as u32).into())
            }
            TypeContent::Struct(fields) => {
                let lowered_fields: Vec<_> = fields
                    .iter()
                    .map(|field| self.lower_type(*field))
                    .collect::<Result<Vec<_>>>()?
                    .into_iter()
                    .map(|lt| lt.as_basic())
                    .collect::<Result<Vec<_>>>()?;
                LoweredType::Basic(self.llvm.struct_type(&lowered_fields, false).into())
            }
            TypeContent::TypedPointer(pointee) => {
                // LLVM opaque pointers discard the pointee; we still type-check the pointee for
                // IR correctness.
                let _ = self.lower_type(*pointee)?;
                LoweredType::Basic(self.llvm.ptr_type(AddressSpace::default()).into())
            }
            TypeContent::Pointer => LoweredType::Basic(
                self.llvm
                    .ptr_type(AddressSpace::default())
                    .as_basic_type_enum(),
            ),
            other => {
                return Err(LlvmError::UnsupportedType(format!(
                    "lowering for type {:?} not implemented",
                    other
                )))
            }
        };

        self.type_cache.insert(ty, lowered);
        Ok(lowered)
    }
}

#[derive(Clone, Copy, Debug)]
enum LoweredType<'ctx> {
    Void,
    Basic(BasicTypeEnum<'ctx>),
}

impl<'ctx> LoweredType<'ctx> {
    fn as_basic(self) -> Result<BasicTypeEnum<'ctx>> {
        match self {
            LoweredType::Void => Err(LlvmError::UnsupportedType(
                "void cannot be used in this position".into(),
            )),
            LoweredType::Basic(b) => Ok(b),
        }
    }
}
