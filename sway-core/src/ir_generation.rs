pub(crate) mod compile;
pub mod const_eval;
mod convert;
mod function;
mod lexical_map;
mod purity;
pub mod storage;
mod types;

use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hasher},
};

use sway_error::error::CompileError;
use sway_features::ExperimentalFeatures;
use sway_ir::{
    Backtrace, Context, Function, InstOp, InstructionInserter, IrError, Kind, Module, Type, Value,
};
use sway_types::{span::Span, Ident};

pub(crate) use purity::{check_function_purity, PurityEnv};

use crate::{
    engine_threading::HashWithEngines,
    ir_generation::function::FnCompiler,
    language::ty::{self, TyCodeBlock, TyExpression, TyFunctionDecl, TyReassignmentTarget},
    metadata::MetadataManager,
    types::{LogId, MessageId},
    Engines, PanicOccurrences, PanickingCallOccurrences, TypeId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FnKey(u64);

impl FnKey {
    fn new(decl: &TyFunctionDecl, engines: &Engines) -> Self {
        let mut hasher = DefaultHasher::default();
        decl.hash(&mut hasher, engines);
        let key = hasher.finish();

        Self(key)
    }
}

/// Groups a [TyFunctionDecl] with its [FnKey].
pub(crate) struct KeyedTyFunctionDecl<'a> {
    key: FnKey,
    decl: &'a TyFunctionDecl,
}

impl<'a> KeyedTyFunctionDecl<'a> {
    fn new(decl: &'a TyFunctionDecl, engines: &'a Engines) -> Self {
        Self {
            key: FnKey::new(decl, engines),
            decl,
        }
    }
}

/// Every compiled function needs to go through this cache for two reasons:
/// 1. to have its IR name unique;
/// 2. to avoid being compiled twice.
#[derive(Default)]
pub(crate) struct CompiledFunctionCache {
    cache: HashMap<FnKey, Function>,
}

impl CompiledFunctionCache {
    #[allow(clippy::too_many_arguments)]
    fn get_compiled_function(
        &mut self,
        engines: &Engines,
        context: &mut Context,
        module: Module,
        md_mgr: &mut MetadataManager,
        keyed_decl: &KeyedTyFunctionDecl,
        logged_types_map: &HashMap<TypeId, LogId>,
        messages_types_map: &HashMap<TypeId, MessageId>,
        panic_occurrences: &mut PanicOccurrences,
        panicking_call_occurrences: &mut PanickingCallOccurrences,
        panicking_fn_cache: &mut PanickingFunctionCache,
    ) -> Result<Function, CompileError> {
        let fn_key = keyed_decl.key;
        let decl = keyed_decl.decl;

        let new_callee = match self.cache.get(&fn_key) {
            Some(func) => *func,
            None => {
                let name = Ident::new(Span::from_string(format!(
                    "{}_{}",
                    decl.name,
                    context.get_unique_symbol_id()
                )));
                let callee_fn_decl = ty::TyFunctionDecl {
                    type_parameters: Vec::new(),
                    name,
                    parameters: decl.parameters.clone(),
                    ..decl.clone()
                };
                // Entry functions are already compiled at the top level
                // when compiling scripts, predicates, contracts, and libraries.
                let is_entry = false;
                let is_original_entry = callee_fn_decl.is_main() || callee_fn_decl.is_test();
                let new_func = compile::compile_function(
                    engines,
                    context,
                    md_mgr,
                    module,
                    &callee_fn_decl,
                    &decl.name,
                    FnCompiler::fn_abi_errors_display(decl, engines),
                    logged_types_map,
                    messages_types_map,
                    panic_occurrences,
                    panicking_call_occurrences,
                    panicking_fn_cache,
                    is_entry,
                    is_original_entry,
                    None,
                    self,
                )
                .map_err(|mut x| x.pop().unwrap())?
                .unwrap();

                self.cache.insert(fn_key, new_func);

                new_func
            }
        };

        Ok(new_callee)
    }
}

#[derive(Default)]
pub(crate) struct PanickingFunctionCache {
    cache: HashMap<FnKey, bool>,
}

impl PanickingFunctionCache {
    /// Returns `true` if the function represented by `keyed_decl` can panic.
    ///
    /// By definition, a function can panic, and have the `__backtrace` argument
    /// added, *if it is not an entry or original entry* and if it contains a
    /// `panic` expression, or calls functions that contain `panic` expressions,
    /// recursively.
    ///
    /// Note that "can panic" is purely an IR concept that does not exist in the AST.
    /// The reason is, because we don't have a language, frontend, concept of "can panic",
    /// that we can check during the type checking phase. This would require an attribute
    /// or a similar mechanism to mark functions as "can panic", which we do not want to
    /// have.
    ///
    /// Because of this, we can cannot check during the type checking phase if a
    /// generic function can panic. E.g., in the below example, `foo` needs to be
    /// monomorphized to check if it can panic, and "can panic" can be different
    /// for different monomorphized versions of the function:
    ///
    /// ```sway
    /// fn foo<T>() where T: DoSomething {
    ///     T::do_something();
    /// }
    /// ```
    pub(crate) fn can_panic(
        &mut self,
        keyed_decl: &KeyedTyFunctionDecl,
        engines: &Engines,
    ) -> bool {
        let fn_key = keyed_decl.key;
        let decl = keyed_decl.decl;

        // Function must not be an entry or original entry (test or main).
        if !decl.is_default() {
            return false;
        }

        match self.cache.get(&fn_key) {
            Some(can_panic) => *can_panic,
            None => {
                let can_panic = self.can_code_block_panic(&decl.body, engines);
                self.cache.insert(fn_key, can_panic);
                can_panic
            }
        }
    }

    fn can_code_block_panic(&mut self, body: &TyCodeBlock, engines: &Engines) -> bool {
        for node in body.contents.iter() {
            use ty::TyAstNodeContent::*;
            match &node.content {
                Declaration(ty_decl) => {
                    if let ty::TyDecl::VariableDecl(var_decl) = ty_decl {
                        if self.can_expression_panic(&var_decl.body, engines) {
                            return true;
                        }
                    }
                }
                Expression(expr) => {
                    if self.can_expression_panic(expr, engines) {
                        return true;
                    }
                }
                SideEffect(_) | Error(_, _) => {}
            }
        }

        false
    }

    fn can_expression_panic(&mut self, expr: &TyExpression, engines: &Engines) -> bool {
        use ty::TyExpressionVariant::*;
        match &expr.expression {
            // `Panic` panics by definition.
            Panic(_) => true,

            // `FunctionApplication` can panic if the callee can panic.
            FunctionApplication { fn_ref, .. } => {
                let decl = engines.de().get_function(fn_ref.id());
                let keyed_decl = KeyedTyFunctionDecl::new(&decl, engines);
                // TODO: Add support for recursive functions once https://github.com/FuelLabs/sway/issues/3018 gets developed.
                self.can_panic(&keyed_decl, engines)
            }

            // Expressions with a single expression that could panic.
            MatchExp {
                desugared: expr, ..
            }
            | StructFieldAccess { prefix: expr, .. }
            | TupleElemAccess { prefix: expr, .. }
            | AbiCast { address: expr, .. }
            | EnumTag { exp: expr }
            | UnsafeDowncast { exp: expr, .. }
            | ForLoop { desugared: expr }
            | ImplicitReturn(expr)
            | Return(expr)
            | Ref(expr)
            | Deref(expr) => self.can_expression_panic(expr, engines),

            // Expressions with multiple sub-expressions that could panic.
            LazyOperator { lhs, rhs, .. } => {
                self.can_expression_panic(lhs, engines) || self.can_expression_panic(rhs, engines)
            }
            Tuple { fields } => fields
                .iter()
                .any(|field| self.can_expression_panic(field, engines)),
            ArrayExplicit { contents, .. } => contents
                .iter()
                .any(|elem| self.can_expression_panic(elem, engines)),
            ArrayRepeat { value, length, .. } => {
                self.can_expression_panic(value, engines)
                    || self.can_expression_panic(length, engines)
            }
            ArrayIndex { prefix, index } => {
                self.can_expression_panic(prefix, engines)
                    || self.can_expression_panic(index, engines)
            }
            StructExpression { fields, .. } => fields
                .iter()
                .any(|field| self.can_expression_panic(&field.value, engines)),
            IfExp {
                condition,
                then,
                r#else,
            } => {
                self.can_expression_panic(condition, engines)
                    || self.can_expression_panic(then, engines)
                    || r#else
                        .as_ref()
                        .map_or(false, |r#else| self.can_expression_panic(r#else, engines))
            }
            AsmExpression { registers, .. } => registers.iter().any(|reg| {
                reg.initializer
                    .as_ref()
                    .is_some_and(|init| self.can_expression_panic(init, engines))
            }),
            EnumInstantiation { contents, .. } => contents
                .as_ref()
                .is_some_and(|contents| self.can_expression_panic(contents, engines)),
            WhileLoop { condition, body } => {
                self.can_expression_panic(condition, engines)
                    || self.can_code_block_panic(body, engines)
            }
            Reassignment(reassignment) => match &reassignment.lhs {
                TyReassignmentTarget::ElementAccess { indices, .. } => {
                    indices.iter().any(|index| match index {
                        ty::ProjectionKind::StructField { .. }
                        | ty::ProjectionKind::TupleField { .. } => false,
                        ty::ProjectionKind::ArrayIndex { index, .. } => {
                            self.can_expression_panic(index, engines)
                        }
                    })
                }
                TyReassignmentTarget::DerefAccess { exp, indices } => {
                    self.can_expression_panic(exp, engines)
                        || indices.iter().any(|index| match index {
                            ty::ProjectionKind::StructField { .. }
                            | ty::ProjectionKind::TupleField { .. } => false,
                            ty::ProjectionKind::ArrayIndex { index, .. } => {
                                self.can_expression_panic(index, engines)
                            }
                        })
                }
            },

            CodeBlock(block) => self.can_code_block_panic(block, engines),

            // Expressions that cannot panic.
            Literal(_)
            | ConstantExpression { .. }
            | ConfigurableExpression { .. }
            | ConstGenericExpression { .. }
            | VariableExpression { .. }
            | FunctionParameter
            | StorageAccess(_)
            | IntrinsicFunction(_)
            | AbiName(_)
            | Break
            | Continue => false,
        }
    }
}

pub fn compile_program<'a>(
    program: &ty::TyProgram,
    panic_occurrences: &'a mut PanicOccurrences,
    panicking_call_occurrences: &'a mut PanickingCallOccurrences,
    include_tests: bool,
    engines: &'a Engines,
    experimental: ExperimentalFeatures,
    backtrace: Backtrace,
) -> Result<Context<'a>, Vec<CompileError>> {
    let declaration_engine = engines.de();

    let test_fns = match include_tests {
        true => program.test_fns(declaration_engine).collect(),
        false => vec![],
    };

    let ty::TyProgram {
        kind,
        namespace,
        logged_types,
        messages_types,
        declarations,
        ..
    } = program;

    let logged_types = logged_types
        .iter()
        .map(|(log_id, type_id)| (*type_id, *log_id))
        .collect();

    let messages_types = messages_types
        .iter()
        .map(|(message_id, type_id)| (*type_id, *message_id))
        .collect();

    let mut ctx = Context::new(engines.se(), experimental, backtrace);
    ctx.program_kind = match kind {
        ty::TyProgramKind::Script { .. } => Kind::Script,
        ty::TyProgramKind::Predicate { .. } => Kind::Predicate,
        ty::TyProgramKind::Contract { .. } => Kind::Contract,
        ty::TyProgramKind::Library { .. } => Kind::Library,
    };

    let mut compiled_fn_cache = CompiledFunctionCache::default();
    let mut panicking_fn_cache = PanickingFunctionCache::default();

    match kind {
        // Predicates and scripts have the same codegen, their only difference is static
        // type-check time checks.
        ty::TyProgramKind::Script { entry_function, .. } => compile::compile_script(
            engines,
            &mut ctx,
            entry_function,
            namespace,
            &logged_types,
            &messages_types,
            panic_occurrences,
            panicking_call_occurrences,
            &mut panicking_fn_cache,
            &test_fns,
            &mut compiled_fn_cache,
        ),
        ty::TyProgramKind::Predicate { entry_function, .. } => compile::compile_predicate(
            engines,
            &mut ctx,
            entry_function,
            namespace,
            &logged_types,
            &messages_types,
            panic_occurrences,
            panicking_call_occurrences,
            &mut panicking_fn_cache,
            &test_fns,
            &mut compiled_fn_cache,
        ),
        ty::TyProgramKind::Contract {
            entry_function,
            abi_entries,
        } => compile::compile_contract(
            &mut ctx,
            entry_function.as_ref(),
            abi_entries,
            namespace,
            declarations,
            &logged_types,
            &messages_types,
            panic_occurrences,
            panicking_call_occurrences,
            &mut panicking_fn_cache,
            &test_fns,
            engines,
            &mut compiled_fn_cache,
        ),
        ty::TyProgramKind::Library { .. } => compile::compile_library(
            engines,
            &mut ctx,
            namespace,
            &logged_types,
            &messages_types,
            panic_occurrences,
            panicking_call_occurrences,
            &mut panicking_fn_cache,
            &test_fns,
            &mut compiled_fn_cache,
        ),
    }?;

    type_correction(&mut ctx).map_err(|ir_error: sway_ir::IrError| {
        vec![CompileError::InternalOwned(
            ir_error.to_string(),
            Span::dummy(),
        )]
    })?;

    ctx.verify().map_err(|ir_error: sway_ir::IrError| {
        vec![CompileError::InternalOwned(
            ir_error.to_string(),
            Span::dummy(),
        )]
    })
}

fn type_correction(ctx: &mut Context) -> Result<(), IrError> {
    struct TypeCorrection {
        actual_ty: sway_ir::Type,
        expected_ty: sway_ir::Type,
        use_instr: sway_ir::Value,
        use_idx: usize,
    }
    // This is a copy of sway_core::asm_generation::fuel::fuel_asm_builder::FuelAsmBuilder::is_copy_type.
    fn is_copy_type(ty: &Type, context: &Context) -> bool {
        ty.is_unit(context)
            || ty.is_never(context)
            || ty.is_bool(context)
            || ty.is_ptr(context)
            || ty.get_uint_width(context).map(|x| x < 256).unwrap_or(false)
    }

    let mut instrs_to_fix = Vec::new();
    for module in ctx.module_iter() {
        for function in module.function_iter(ctx) {
            for (_block, instr) in function.instruction_iter(ctx).collect::<Vec<_>>() {
                match &instr.get_instruction(ctx).unwrap().op {
                    InstOp::Call(callee, actual_params) => {
                        let formal_params: Vec<_> = callee.args_iter(ctx).collect();
                        for (param_idx, (actual_param, (_, formal_param))) in
                            actual_params.iter().zip(formal_params.iter()).enumerate()
                        {
                            let actual_ty = actual_param.get_type(ctx).unwrap();
                            let formal_ty = formal_param.get_type(ctx).unwrap();
                            if actual_ty != formal_ty {
                                instrs_to_fix.push(TypeCorrection {
                                    actual_ty,
                                    expected_ty: formal_ty,
                                    use_instr: instr,
                                    use_idx: param_idx,
                                });
                            }
                        }
                    }
                    InstOp::AsmBlock(_block, _args) => {
                        // Non copy type args to asm blocks are passed by reference.
                        let op = &instr.get_instruction(ctx).unwrap().op;
                        let args = op
                            .get_operands()
                            .iter()
                            .enumerate()
                            .map(|(idx, init)| (idx, init.get_type(ctx).unwrap()))
                            .collect::<Vec<_>>();
                        for (arg_idx, arg_ty) in args {
                            if !is_copy_type(&arg_ty, ctx) {
                                instrs_to_fix.push(TypeCorrection {
                                    actual_ty: arg_ty,
                                    expected_ty: Type::new_typed_pointer(ctx, arg_ty),
                                    use_instr: instr,
                                    use_idx: arg_idx,
                                });
                            }
                        }
                    }
                    InstOp::GetElemPtr {
                        base,
                        elem_ptr_ty,
                        indices,
                    } => {
                        let base_ty = base.get_type(ctx).unwrap();
                        if let (Some(base_pointee_ty), Some(elem_inner_ty)) = (
                            base_ty.get_pointee_type(ctx),
                            elem_ptr_ty.get_pointee_type(ctx),
                        ) {
                            // The base is a pointer type. We need to see if it's a double pointer.
                            if let Some(base_pointee_pointee_ty) =
                                base_pointee_ty.get_pointee_type(ctx)
                            {
                                // We have a double pointer. If just loading once solves our problem, we do that.
                                let indexed_ty =
                                    base_pointee_pointee_ty.get_value_indexed_type(ctx, indices);
                                if indexed_ty.is_some_and(|ty| ty == elem_inner_ty) {
                                    instrs_to_fix.push(TypeCorrection {
                                        actual_ty: base_ty,
                                        expected_ty: base_pointee_ty,
                                        use_instr: instr,
                                        use_idx: indices.len(),
                                    });
                                }
                            }
                        } else {
                            // The base is not a pointer type. If a pointer to base_ty works for us, do that.
                            let elem_ptr_ty = *elem_ptr_ty;
                            let indices = indices.clone(); // Cloning needed because of mutable and immutable borrow of `ctx`.
                            let pointer_to_base = Type::new_typed_pointer(ctx, base_ty);
                            if pointer_to_base.get_value_indexed_type(ctx, &indices)
                                == Some(elem_ptr_ty)
                            {
                                instrs_to_fix.push(TypeCorrection {
                                    actual_ty: base_ty,
                                    expected_ty: pointer_to_base,
                                    use_instr: instr,
                                    use_idx: indices.len(),
                                });
                            }
                        }
                    }
                    InstOp::Store {
                        dst_val_ptr,
                        stored_val,
                    } => {
                        let dst_ty = dst_val_ptr.get_type(ctx).unwrap();
                        let stored_ty = stored_val.get_type(ctx).unwrap();
                        if let Some(dst_pointee_ty) = dst_ty.get_pointee_type(ctx) {
                            // The destination is a pointer type. We need to see if it's a double pointer.
                            if let Some(dst_pointee_pointee_ty) =
                                dst_pointee_ty.get_pointee_type(ctx)
                            {
                                // We have a double pointer. If just loading once solves our problem, we do that.
                                if dst_pointee_pointee_ty == stored_ty {
                                    instrs_to_fix.push(TypeCorrection {
                                        actual_ty: dst_ty,
                                        expected_ty: dst_pointee_ty,
                                        use_instr: instr,
                                        use_idx: 0,
                                    });
                                }
                            } else if let Some(stored_pointee_ty) = stored_ty.get_pointee_type(ctx)
                            {
                                // The value being stored is a pointer to what should've been stored.
                                // So we just load the value and store it.
                                if dst_pointee_ty == stored_pointee_ty {
                                    instrs_to_fix.push(TypeCorrection {
                                        actual_ty: stored_ty,
                                        expected_ty: stored_pointee_ty,
                                        use_instr: instr,
                                        use_idx: 1,
                                    });
                                }
                            }
                        } else {
                            // The destination is not a pointer type, but should've been.
                            let pointer_to_dst = Type::new_typed_pointer(ctx, dst_ty);
                            if pointer_to_dst == stored_ty {
                                instrs_to_fix.push(TypeCorrection {
                                    actual_ty: dst_ty,
                                    expected_ty: pointer_to_dst,
                                    use_instr: instr,
                                    use_idx: 0,
                                });
                            }
                        }
                    }
                    InstOp::Ret(ret_val, ret_ty) => {
                        if let Some(ret_val_pointee_ty) = ret_val
                            .get_type(ctx)
                            .and_then(|ret_val_ty| ret_val_ty.get_pointee_type(ctx))
                        {
                            if ret_val_pointee_ty == *ret_ty {
                                instrs_to_fix.push(TypeCorrection {
                                    actual_ty: ret_val.get_type(ctx).unwrap(),
                                    expected_ty: *ret_ty,
                                    use_instr: instr,
                                    use_idx: 0,
                                });
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    for TypeCorrection {
        actual_ty,
        expected_ty,
        use_instr,
        use_idx,
    } in instrs_to_fix
    {
        let function = use_instr.get_instruction(ctx).unwrap().get_function(ctx);
        if expected_ty
            .get_pointee_type(ctx)
            .is_some_and(|pointee| pointee == actual_ty)
        {
            // The expected type is a pointer to the actual type.
            // If the actual value was just loaded, then we go to the source of the load,
            // otherwise, we store it to a new local and pass the address of that local.
            let actual_use = use_instr.get_instruction(ctx).unwrap().op.get_operands()[use_idx];
            if let Some(InstOp::Load(src_ptr)) = actual_use.get_instruction(ctx).map(|i| &i.op) {
                let src_ptr = *src_ptr;
                use_instr
                    .get_instruction_mut(ctx)
                    .unwrap()
                    .op
                    .set_operand(src_ptr, use_idx);
            } else {
                let parent_block = use_instr.get_instruction(ctx).unwrap().parent;
                let new_local = function.new_unique_local_var(
                    ctx,
                    "type_fix".to_string(),
                    actual_ty,
                    None,
                    true,
                );
                let new_local =
                    Value::new_instruction(ctx, parent_block, InstOp::GetLocal(new_local));
                let store = Value::new_instruction(
                    ctx,
                    parent_block,
                    InstOp::Store {
                        dst_val_ptr: new_local,
                        stored_val: actual_use,
                    },
                );
                let mut inserter = InstructionInserter::new(
                    ctx,
                    parent_block,
                    sway_ir::InsertionPosition::Before(use_instr),
                );
                inserter.insert_slice(&[new_local, store]);
                // Update the use instruction to use the new local
                use_instr
                    .get_instruction_mut(ctx)
                    .unwrap()
                    .op
                    .set_operand(new_local, use_idx);
            }
        } else if actual_ty
            .get_pointee_type(ctx)
            .is_some_and(|pointee| pointee == expected_ty)
        {
            // Just load the actual value.
            let load = Value::new_instruction(
                ctx,
                use_instr.get_instruction(ctx).unwrap().parent,
                InstOp::Load(use_instr.get_instruction(ctx).unwrap().op.get_operands()[use_idx]),
            );
            let mut inserter = InstructionInserter::new(
                ctx,
                use_instr.get_instruction(ctx).unwrap().parent,
                sway_ir::InsertionPosition::Before(use_instr),
            );
            inserter.insert_slice(&[load]);
            // Update the use instruction to use the new load
            use_instr
                .get_instruction_mut(ctx)
                .unwrap()
                .op
                .set_operand(load, use_idx);
        }
    }
    Ok(())
}
