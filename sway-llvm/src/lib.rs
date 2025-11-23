//! Experimental LLVM backend for Sway IR using Inkwell.
//!
//! This is the initial scaffolding for translating `sway-ir` modules into LLVM IR. The goal is to
//! grow this incrementally: start with type lowering and function declarations, then add
//! instruction lowering in tiers.

use std::collections::HashMap;
use std::convert::TryInto;

use inkwell::{
    basic_block::BasicBlock,
    builder::{Builder, BuilderError},
    context::Context as LlvmContext,
    module::Module as LlvmModule,
    targets::TargetTriple,
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum},
    values::{
        BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, IntValue, PointerValue,
        ValueKind,
    },
    AddressSpace, IntPredicate,
};
use sway_ir::{
    block::Block,
    constant::{Constant, ConstantContent, ConstantValue},
    function::Function,
    instruction::{BinaryOpKind, BranchToWithArgs, InstOp, Predicate},
    irtype::{Type, TypeContent},
    module::Module,
    value::Value,
    variable::LocalVar,
};
use sway_types::u256::U256;
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
    builder: Builder<'ctx>,
    _opts: BackendOptions,
    type_cache: HashMap<Type, LoweredType<'ctx>>,
    func_map: HashMap<Function, inkwell::values::FunctionValue<'ctx>>,
    value_map: HashMap<Value, BasicValueEnum<'ctx>>,
    block_map: HashMap<Block, BasicBlock<'ctx>>,
    local_allocas: HashMap<LocalVar, PointerValue<'ctx>>,
    block_arg_phis: HashMap<(Block, usize), inkwell::values::PhiValue<'ctx>>,
    current_block: Option<BasicBlock<'ctx>>,
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
            builder: llvm.create_builder(),
            _opts: opts,
            type_cache: HashMap::new(),
            func_map: HashMap::new(),
            value_map: HashMap::new(),
            block_map: HashMap::new(),
            local_allocas: HashMap::new(),
            block_arg_phis: HashMap::new(),
            current_block: None,
        })
    }

    /// For now we only emit function declarations. Bodies and instruction lowering will follow.
    fn lower_module_decls(&mut self) -> Result<()> {
        for func in self.ir_module.function_iter(self.ir) {
            let fn_val = self.declare_function(func)?;
            self.map_function_arguments(func, fn_val)?;
            self.map_function_blocks(func, fn_val)?;
            self.func_map.insert(func, fn_val);
        }
        self.lower_function_bodies()?;
        self.create_main_shim()?;
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

    fn map_function_arguments(
        &mut self,
        func: Function,
        fn_val: FunctionValue<'ctx>,
    ) -> Result<()> {
        for (idx, (_, arg_val)) in func.args_iter(self.ir).enumerate() {
            let param = fn_val
                .get_nth_param(idx as u32)
                .ok_or_else(|| LlvmError::Lowering("missing function parameter".into()))?;
            self.value_map.insert(*arg_val, param);
        }
        Ok(())
    }

    fn map_function_blocks(&mut self, func: Function, fn_val: FunctionValue<'ctx>) -> Result<()> {
        for block in func.block_iter(self.ir) {
            let label = block.get_label(self.ir);
            let bb = self.llvm.append_basic_block(fn_val, &label);
            self.block_map.insert(block, bb);
        }
        Ok(())
    }

    fn lower_function_bodies(&mut self) -> Result<()> {
        for func in self.ir_module.function_iter(self.ir) {
            self.create_local_allocas(func)?;

            for block in func.block_iter(self.ir) {
                self.create_block_phis(block)?;
            }

            for block in func.block_iter(self.ir) {
                let bb = *self
                    .block_map
                    .get(&block)
                    .ok_or_else(|| LlvmError::Lowering("missing block mapping".into()))?;
                self.current_block = Some(bb);
                self.builder.position_at_end(bb);
                for inst_value in block.instruction_iter(self.ir) {
                    self.lower_instruction(inst_value)?;
                }
            }

            self.current_block = None;
        }
        Ok(())
    }

    fn create_main_shim(&mut self) -> Result<()> {
        let entry_func = match self.find_entry_function() {
            Some(func) => func,
            None => return Ok(()),
        };

        let entry_val = *self.func_map.get(&entry_func).ok_or_else(|| {
            LlvmError::Lowering("missing LLVM function for entry point".into())
        })?;

        let main_ret = self.llvm.i32_type();
        let main_type = main_ret.fn_type(&[], false);
        let main_fn = self.llvm_module.add_function("main", main_type, None);
        let bb = self.llvm.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(bb);
        self.handle_builder_result(self.builder.build_call(entry_val, &[], "call_entry"))?;
        let zero = main_ret.const_zero();
        self.handle_builder_result(self.builder.build_return(Some(&zero)))?;
        Ok(())
    }

    fn find_entry_function(&self) -> Option<Function> {
        let mut iter = self.ir_module.function_iter(self.ir);
        if let Some(func) = iter.find(|func| func.is_original_entry(self.ir)) {
            return Some(func);
        }
        self.ir_module
            .function_iter(self.ir)
            .find(|func| func.is_entry(self.ir))
    }

    fn create_block_phis(&mut self, block: Block) -> Result<()> {
        let bb = *self
            .block_map
            .get(&block)
            .ok_or_else(|| LlvmError::Lowering("missing block mapping for phi creation".into()))?;

        if let Some(first_inst) = bb.get_first_instruction() {
            self.builder.position_before(&first_inst);
        } else {
            self.builder.position_at_end(bb);
        }

        let pred_count = block.num_predecessors(self.ir);
        for (idx, arg_val) in block.arg_iter(self.ir).enumerate() {
            let arg_ty = arg_val
                .get_type(self.ir)
                .ok_or_else(|| LlvmError::Lowering("block argument missing type".into()))?;
            let lowered = self.lower_basic_metadata_type(arg_ty)?;
            let basic = match lowered {
                BasicMetadataTypeEnum::ArrayType(t) => t.as_basic_type_enum(),
                BasicMetadataTypeEnum::FloatType(t) => t.as_basic_type_enum(),
                BasicMetadataTypeEnum::IntType(t) => t.as_basic_type_enum(),
                BasicMetadataTypeEnum::PointerType(t) => t.as_basic_type_enum(),
                BasicMetadataTypeEnum::StructType(t) => t.as_basic_type_enum(),
                BasicMetadataTypeEnum::VectorType(t) => t.as_basic_type_enum(),
                BasicMetadataTypeEnum::ScalableVectorType(t) => t.as_basic_type_enum(),
                BasicMetadataTypeEnum::MetadataType(_) => {
                    return Err(LlvmError::UnsupportedType(
                        "metadata type not supported in block arg".into(),
                    ))
                }
            };

            if pred_count == 0 {
                let undef = basic.const_zero();
                self.value_map.insert(*arg_val, undef);
                continue;
            }

            let phi = self
                .builder
                .build_phi(basic, &format!("arg_{}", idx))
                .map_err(|e| LlvmError::Lowering(format!("phi build error: {e}")))?;
            self.block_arg_phis.insert((block, idx), phi);
            self.value_map.insert(*arg_val, phi.as_basic_value());
        }

        self.builder.position_at_end(bb);
        Ok(())
    }

    fn create_local_allocas(&mut self, func: Function) -> Result<()> {
        let entry_block = func.get_entry_block(self.ir);
        let entry_bb = *self
            .block_map
            .get(&entry_block)
            .ok_or_else(|| LlvmError::Lowering("missing entry block".into()))?;
        self.builder.position_at_end(entry_bb);
        for (name, local_var) in func.locals_iter(self.ir) {
            let inner_ty = local_var.get_inner_type(self.ir);
            let lowered = self.lower_type(inner_ty)?.as_basic()?;
            let alloca = self.handle_builder_result(self.builder.build_alloca(lowered, name))?;
            if let Some(initializer) = local_var.get_initializer(self.ir) {
                let init_val = self.lower_constant(*initializer)?;
                self.handle_builder_result(self.builder.build_store(alloca, init_val))?;
            }
            self.local_allocas.insert(*local_var, alloca);
        }
        Ok(())
    }

    fn lower_instruction(&mut self, inst_value: Value) -> Result<()> {
        let instruction = inst_value
            .get_instruction(self.ir)
            .ok_or_else(|| LlvmError::Lowering("value has no instruction".into()))?;
        match &instruction.op {
            InstOp::Nop => Ok(()),
            InstOp::GetLocal(local_var) => {
                let ptr = *self
                    .local_allocas
                    .get(local_var)
                    .ok_or_else(|| LlvmError::Lowering("missing local pointer".into()))?;
                self.value_map.insert(inst_value, ptr.as_basic_value_enum());
                Ok(())
            }
            InstOp::Load(ptr_val) => {
                let ptr_val_basic = self.get_basic_value(*ptr_val)?;
                let ptr = self.to_pointer_value(ptr_val_basic)?;
                let load_ty = inst_value
                    .get_type(self.ir)
                    .ok_or_else(|| LlvmError::Lowering("load value missing type".into()))?;
                let lowered_load_ty = self.lower_type(load_ty)?.as_basic()?;
                let load = self.handle_builder_result(self.builder.build_load(
                    lowered_load_ty,
                    ptr,
                    "load",
                ))?;
                self.value_map.insert(inst_value, load);
                Ok(())
            }
            InstOp::Store {
                dst_val_ptr,
                stored_val,
            } => {
                let ptr_val_basic = self.get_basic_value(*dst_val_ptr)?;
                let ptr = self.to_pointer_value(ptr_val_basic)?;
                let val = self.get_basic_value(*stored_val)?;
                self.handle_builder_result(self.builder.build_store(ptr, val))?;
                Ok(())
            }
            InstOp::GetElemPtr { base, indices, .. } => {
                self.lower_gep(inst_value, *base, indices)?;
                Ok(())
            }
            InstOp::BinaryOp { op, arg1, arg2 } => {
                let lhs_basic = self.get_basic_value(*arg1)?;
                let lhs = self.ensure_int_value(lhs_basic)?;
                let rhs_basic = self.get_basic_value(*arg2)?;
                let rhs = self.ensure_int_value(rhs_basic)?;
                let res = match op {
                    BinaryOpKind::Add => {
                        self.handle_builder_result(self.builder.build_int_add(lhs, rhs, "add"))?
                    }
                    BinaryOpKind::Sub => {
                        self.handle_builder_result(self.builder.build_int_sub(lhs, rhs, "sub"))?
                    }
                    BinaryOpKind::Mul => {
                        self.handle_builder_result(self.builder.build_int_mul(lhs, rhs, "mul"))?
                    }
                    BinaryOpKind::Div => self.handle_builder_result(
                        self.builder.build_int_unsigned_div(lhs, rhs, "div"),
                    )?,
                    BinaryOpKind::Mod => self.handle_builder_result(
                        self.builder.build_int_unsigned_rem(lhs, rhs, "mod"),
                    )?,
                    BinaryOpKind::And => {
                        self.handle_builder_result(self.builder.build_and(lhs, rhs, "and"))?
                    }
                    BinaryOpKind::Or => {
                        self.handle_builder_result(self.builder.build_or(lhs, rhs, "or"))?
                    }
                    BinaryOpKind::Xor => {
                        self.handle_builder_result(self.builder.build_xor(lhs, rhs, "xor"))?
                    }
                    BinaryOpKind::Lsh => {
                        self.handle_builder_result(self.builder.build_left_shift(lhs, rhs, "lsh"))?
                    }
                    BinaryOpKind::Rsh => self.handle_builder_result(
                        self.builder.build_right_shift(lhs, rhs, true, "rsh"),
                    )?,
                };
                self.value_map.insert(inst_value, res.as_basic_value_enum());
                Ok(())
            }
            InstOp::Cmp(predicate, lhs, rhs) => {
                let lhs_val_basic = self.get_basic_value(*lhs)?;
                let lhs_val = self.ensure_int_value(lhs_val_basic)?;
                let rhs_val_basic = self.get_basic_value(*rhs)?;
                let rhs_val = self.ensure_int_value(rhs_val_basic)?;
                let pred = match predicate {
                    Predicate::Equal => IntPredicate::EQ,
                    Predicate::LessThan => IntPredicate::ULT,
                    Predicate::GreaterThan => IntPredicate::UGT,
                };
                let cmp = self.handle_builder_result(
                    self.builder
                        .build_int_compare(pred, lhs_val, rhs_val, "cmp"),
                )?;
                self.value_map.insert(inst_value, cmp.as_basic_value_enum());
                Ok(())
            }
            InstOp::Branch(target) => {
                let bb = *self
                    .block_map
                    .get(&target.block)
                    .ok_or_else(|| LlvmError::Lowering("missing branch block".into()))?;
                self.assign_branch_args(target)?;
                self.handle_builder_result(self.builder.build_unconditional_branch(bb))?;
                Ok(())
            }
            InstOp::ConditionalBranch {
                cond_value,
                true_block,
                false_block,
            } => {
                let cond_basic = self.get_basic_value(*cond_value)?;
                let cond = self.ensure_int_value(cond_basic)?;
                let true_bb = *self.block_map.get(&true_block.block).ok_or_else(|| {
                    LlvmError::Lowering("missing true block for conditional branch".into())
                })?;
                let false_bb = *self.block_map.get(&false_block.block).ok_or_else(|| {
                    LlvmError::Lowering("missing false block for conditional branch".into())
                })?;
                self.assign_branch_args(true_block)?;
                self.assign_branch_args(false_block)?;
                self.handle_builder_result(
                    self.builder
                        .build_conditional_branch(cond, true_bb, false_bb),
                )?;
                Ok(())
            }
            InstOp::Ret(ret_val, ty) => {
                if ty.is_unit(self.ir) {
                    self.handle_builder_result(self.builder.build_return(None))?;
                } else {
                    let val = self.get_basic_value(*ret_val)?;
                    self.handle_builder_result(self.builder.build_return(Some(&val)))?;
                }
                Ok(())
            }
            InstOp::MemCopyVal {
                dst_val_ptr,
                src_val_ptr,
            } => {
                let dst_basic = self.get_basic_value(*dst_val_ptr)?;
                let dst_ptr = self.to_pointer_value(dst_basic)?;
                let src_basic = self.get_basic_value(*src_val_ptr)?;
                let src_ptr = self.to_pointer_value(src_basic)?;
                let byte_len = self.get_pointee_size(*dst_val_ptr)?;
                let size_ty = self.llvm.custom_width_int_type(64);
                let size_const = size_ty.const_int(byte_len, false);
                if src_ptr.is_null() {
                    let zero_val = self.llvm.i8_type().const_zero();
                    self.handle_builder_result(
                        self.builder
                            .build_memset(dst_ptr, 8, zero_val, size_const),
                    )?;
                } else {
                    self.handle_builder_result(
                        self.builder
                            .build_memcpy(dst_ptr, 8, src_ptr, 8, size_const),
                    )?;
                }
                Ok(())
            }
            InstOp::AsmBlock(asm_block, _) => {
                if let LoweredType::Basic(basic) = self.lower_type(asm_block.return_type)? {
                    let zero = basic.const_zero().as_basic_value_enum();
                    self.value_map.insert(inst_value, zero);
                }
                Ok(())
            }
            InstOp::CastPtr(value, ty) => {
                let basic_val = self.get_basic_value(*value)?;
                let dest_type = self.lower_type(*ty)?;
                let dest_basic = dest_type.as_basic()?;
                let cast = if dest_basic.is_pointer_type() && basic_val.is_int_value() {
                    let int_val = basic_val.into_int_value();
                    self.handle_builder_result(self.builder.build_int_to_ptr(
                        int_val,
                        dest_basic.into_pointer_type(),
                        "inttoptr",
                    ))?
                    .as_basic_value_enum()
                } else if dest_basic.is_int_type() && basic_val.is_pointer_value() {
                    let ptr_val = basic_val.into_pointer_value();
                    self.handle_builder_result(self.builder.build_ptr_to_int(
                        ptr_val,
                        dest_basic.into_int_type(),
                        "ptrtoint",
                    ))?
                    .as_basic_value_enum()
                } else {
                    self.handle_builder_result(
                        self.builder
                            .build_bit_cast(basic_val, dest_basic, "castptr"),
                    )?
                };
                self.value_map.insert(inst_value, cast);
                Ok(())
            }
            InstOp::Call(target_fn, args) => {
                let function = *self.func_map.get(target_fn).ok_or_else(|| {
                    LlvmError::Lowering("call target function not declared".into())
                })?;
                let metadata_args = args
                    .iter()
                    .map(|arg| {
                        let val = self.get_basic_value(*arg)?;
                        Ok(BasicMetadataValueEnum::from(val))
                    })
                    .collect::<Result<Vec<_>>>()?;
                let call_site = self.handle_builder_result(self.builder.build_call(
                    function,
                    &metadata_args,
                    "call",
                ))?;
                if let ValueKind::Basic(result) = call_site.try_as_basic_value() {
                    self.value_map.insert(inst_value, result);
                }
                Ok(())
            }
            op => {
                eprintln!("LLVM backend unsupported instruction: {:?}", op);
                Err(LlvmError::UnsupportedInstruction(
                    "instruction not implemented",
                ))
            }
        }
    }

    fn get_basic_value(&mut self, value: Value) -> Result<BasicValueEnum<'ctx>> {
        if let Some(val) = self.value_map.get(&value) {
            return Ok(*val);
        }
        if let Some(constant) = value.get_constant(self.ir) {
            let lowered = self.lower_constant(*constant)?;
            self.value_map.insert(value, lowered);
            return Ok(lowered);
        }
        Err(LlvmError::Lowering("value not yet lowered".into()))
    }

    fn to_pointer_value(&self, value: BasicValueEnum<'ctx>) -> Result<PointerValue<'ctx>> {
        Ok(value.into_pointer_value())
    }

    fn ensure_int_value(&mut self, value: BasicValueEnum<'ctx>) -> Result<IntValue<'ctx>> {
        if value.is_int_value() {
            Ok(value.into_int_value())
        } else if value.is_pointer_value() {
            let ptr = value.into_pointer_value();
            let int_ty = self.llvm.custom_width_int_type(64);
            self.handle_builder_result(self.builder.build_ptr_to_int(ptr, int_ty, "ptrtoint"))
        } else {
            Err(LlvmError::Lowering(
                "value is not integer or pointer".into(),
            ))
        }
    }

    fn handle_builder_result<T>(&self, result: std::result::Result<T, BuilderError>) -> Result<T> {
        result.map_err(|err| LlvmError::Lowering(format!("builder error: {}", err)))
    }

    fn assign_branch_args(&mut self, branch: &BranchToWithArgs) -> Result<()> {
        let pred_bb = self.current_block.ok_or_else(|| {
            LlvmError::Lowering("no current block when assigning branch args".into())
        })?;

        let arg_count = branch.block.arg_iter(self.ir).count();
        if arg_count != branch.args.len() {
            return Err(LlvmError::Lowering("branch argument count mismatch".into()));
        }

        for (idx, _) in branch.block.arg_iter(self.ir).enumerate() {
            let incoming = branch.args.get(idx).ok_or_else(|| {
                LlvmError::Lowering("branch argument mismatch".into())
            })?;
            let val = self.get_basic_value(*incoming)?;
            let phi = self
                .block_arg_phis
                .get(&(branch.block, idx))
                .ok_or_else(|| LlvmError::Lowering("missing phi for block argument".into()))?;
            phi.add_incoming(&[(&val, pred_bb)]);
        }

        Ok(())
    }

    fn lower_gep(
        &mut self,
        inst_value: Value,
        base: Value,
        indices: &[Value],
    ) -> Result<()> {
        let base_basic = self.get_basic_value(base)?;
        let ptr = self.to_pointer_value(base_basic)?;
        let base_ty = base
            .get_type(self.ir)
            .ok_or_else(|| LlvmError::Lowering("gep base missing type".into()))?;
        let pointee_ty = base_ty
            .get_pointee_type(self.ir)
            .ok_or_else(|| LlvmError::Lowering("gep base is not a pointer".into()))?;
        let lowered_pointee = self.lower_type(pointee_ty)?.as_basic()?;

        let mut llvm_indices: Vec<IntValue<'ctx>> = Vec::with_capacity(indices.len() + 1);
        let i64_ty = self.llvm.custom_width_int_type(64);
        let i32_ty = self.llvm.i32_type();

        match pointee_ty.get_content(self.ir) {
            TypeContent::Array(_, _) if indices.len() == 1 => {
                llvm_indices.push(i64_ty.const_zero());
                let idx_basic = self.get_basic_value(indices[0])?;
                let idx_val = self.ensure_int_value(idx_basic)?;
                llvm_indices.push(idx_val);
            }
            TypeContent::Struct(_) if indices.len() == 1 => {
                llvm_indices.push(i32_ty.const_zero());
                let idx_basic = self.get_basic_value(indices[0])?;
                let idx_val = self.ensure_int_value(idx_basic)?;
                let const_idx = idx_val.get_zero_extended_constant().ok_or_else(|| {
                    LlvmError::Lowering("struct GEP index must be a constant integer".into())
                })?;
                let field_idx: u32 = const_idx.try_into().map_err(|_| {
                    LlvmError::Lowering("struct GEP index too large for i32".into())
                })?;
                llvm_indices.push(i32_ty.const_int(field_idx as u64, false));
            }
            _ => {
                for idx in indices {
                    let idx_basic = self.get_basic_value(*idx)?;
                    let idx_val = self.ensure_int_value(idx_basic)?;
                    llvm_indices.push(idx_val);
                }
            }
        }

        let gep = self.handle_builder_result(unsafe {
            self.builder
                .build_gep(lowered_pointee, ptr, &llvm_indices, "gep")
        })?;
        self.value_map.insert(inst_value, gep.as_basic_value_enum());
        Ok(())
    }

    fn get_pointee_size(&self, ptr_value: Value) -> Result<u64> {
        let ty = ptr_value
            .get_type(self.ir)
            .ok_or_else(|| LlvmError::Lowering("value missing type".into()))?;
        let pointee = ty
            .get_pointee_type(self.ir)
            .ok_or_else(|| LlvmError::Lowering("value is not a pointer".into()))?;
        Ok(pointee.size(self.ir).in_bytes())
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
            TypeContent::Slice => {
                let ptr_ty = self.llvm.ptr_type(AddressSpace::default());
                let len_ty = self.llvm.custom_width_int_type(64);
                let slice_struct = self
                    .llvm
                    .struct_type(
                        &[ptr_ty.as_basic_type_enum(), len_ty.as_basic_type_enum()],
                        false,
                    )
                    .into();
                LoweredType::Basic(slice_struct)
            }
            TypeContent::TypedSlice(item_ty) => {
                let _ = self.lower_type(*item_ty)?;
                let ptr_ty = self.llvm.ptr_type(AddressSpace::default());
                let len_ty = self.llvm.custom_width_int_type(64);
                let slice_struct = self
                    .llvm
                    .struct_type(
                        &[ptr_ty.as_basic_type_enum(), len_ty.as_basic_type_enum()],
                        false,
                    )
                    .into();
                LoweredType::Basic(slice_struct)
            }
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

    fn lower_constant(&mut self, constant: Constant) -> Result<BasicValueEnum<'ctx>> {
        let content = constant.get_content(self.ir);
        self.lower_constant_content(content)
    }

    fn lower_constant_content(
        &mut self,
        content: &ConstantContent,
    ) -> Result<BasicValueEnum<'ctx>> {
        let lowered = self.lower_type(content.ty)?;
        let basic_ty = lowered.as_basic()?;
        match &content.value {
            ConstantValue::Undef => Ok(basic_ty.const_zero()),
            ConstantValue::Unit => Ok(self.llvm.bool_type().const_zero().as_basic_value_enum()),
            ConstantValue::Bool(val) => Ok(self
                .llvm
                .bool_type()
                .const_int(*val as u64, false)
                .as_basic_value_enum()),
            ConstantValue::Uint(val) => {
                let width = content
                    .ty
                    .get_uint_width(self.ir)
                    .ok_or_else(|| LlvmError::Lowering("uint constant has no width".into()))?;
                let int_ty = self.llvm.custom_width_int_type(width.into());
                Ok(int_ty.const_int(*val, false).as_basic_value_enum())
            }
            ConstantValue::U256(val) | ConstantValue::B256(val) => {
                let words = u256_to_words(val);
                let int_ty = self.llvm.custom_width_int_type(256);
                Ok(int_ty
                    .const_int_arbitrary_precision(&words)
                    .as_basic_value_enum())
            }
            ConstantValue::Array(_) => Ok(basic_ty.const_zero()),
            ConstantValue::Struct(_) => Ok(basic_ty.const_zero()),
            ConstantValue::Slice(_) | ConstantValue::RawUntypedSlice(_) => {
                Ok(basic_ty.const_zero())
            }
            ConstantValue::Reference(_) => {
                Err(LlvmError::UnsupportedType(
                    "constant references are not supported yet".into(),
                ))
            }
            _ => Ok(basic_ty.const_zero()),
        }
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

fn u256_to_words(value: &U256) -> Vec<u64> {
    let mut bytes = value.to_be_bytes();
    bytes.reverse();
    bytes
        .chunks_exact(8)
        .map(|chunk| {
            let arr: [u8; 8] = chunk.try_into().unwrap();
            u64::from_le_bytes(arr)
        })
        .collect()
}
