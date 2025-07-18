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
use sway_ir::{Context, Function, InstOp, InstructionInserter, IrError, Kind, Module, Type, Value};
use sway_types::{span::Span, Ident};

pub(crate) use purity::{check_function_purity, PurityEnv};

use crate::{
    engine_threading::HashWithEngines,
    language::ty,
    metadata::MetadataManager,
    types::{LogId, MessageId},
    Engines, PanicOccurrences, TypeId,
};

type FnKey = u64;

/// Every compiled function needs to go through this cache for two reasons:
/// 1 - to have its IR name unique;
/// 2 - to avoid being compiled twice.
#[derive(Default)]
pub(crate) struct CompiledFunctionCache {
    recreated_fns: HashMap<FnKey, Function>,
}

impl CompiledFunctionCache {
    #[allow(clippy::too_many_arguments)]
    fn ty_function_decl_to_unique_function(
        &mut self,
        engines: &Engines,
        context: &mut Context,
        module: Module,
        md_mgr: &mut MetadataManager,
        decl: &ty::TyFunctionDecl,
        logged_types_map: &HashMap<TypeId, LogId>,
        messages_types_map: &HashMap<TypeId, MessageId>,
        panic_occurrences: &mut PanicOccurrences,
    ) -> Result<Function, CompileError> {
        // The compiler inlines everything very lazily.  Function calls include the body of the
        // callee (i.e., the callee_body arg above). Library functions are provided in an initial
        // namespace from Forc and when the parser builds the AST (or is it during type checking?)
        // these function bodies are embedded.
        //
        // Here we build little single-use instantiations of the callee and then call them.  Naming
        // is not yet absolute so we must ensure the function names are unique.
        //
        // Eventually we need to Do It Properly and inline into the AST only when necessary, and
        // compile the standard library to an actual module.
        //
        // Get the callee from the cache if we've already compiled it.  We can't insert it with
        // .entry() since `compile_function()` returns a Result we need to handle.  The key to our
        // cache, to uniquely identify a function instance, is the span and the type IDs of any
        // args and type parameters.  It's using the Sway types rather than IR types, which would
        // be more accurate but also more fiddly.

        let mut hasher = DefaultHasher::default();
        decl.hash(&mut hasher, engines);
        let fn_key = hasher.finish();

        let (fn_key, item) = (Some(fn_key), self.recreated_fns.get(&fn_key).copied());
        let new_callee = match item {
            Some(func) => func,
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
                    logged_types_map,
                    messages_types_map,
                    panic_occurrences,
                    is_entry,
                    is_original_entry,
                    None,
                    self,
                )
                .map_err(|mut x| x.pop().unwrap())?
                .unwrap();

                if let Some(fn_key) = fn_key {
                    self.recreated_fns.insert(fn_key, new_func);
                }

                new_func
            }
        };

        Ok(new_callee)
    }
}

pub fn compile_program<'a>(
    program: &ty::TyProgram,
    panic_occurrences: &'a mut PanicOccurrences,
    include_tests: bool,
    engines: &'a Engines,
    experimental: ExperimentalFeatures,
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

    let mut ctx = Context::new(engines.se(), experimental);
    ctx.program_kind = match kind {
        ty::TyProgramKind::Script { .. } => Kind::Script,
        ty::TyProgramKind::Predicate { .. } => Kind::Predicate,
        ty::TyProgramKind::Contract { .. } => Kind::Contract,
        ty::TyProgramKind::Library { .. } => Kind::Library,
    };

    let mut cache = CompiledFunctionCache::default();

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
            &test_fns,
            &mut cache,
        ),
        ty::TyProgramKind::Predicate { entry_function, .. } => compile::compile_predicate(
            engines,
            &mut ctx,
            entry_function,
            namespace,
            &logged_types,
            &messages_types,
            panic_occurrences,
            &test_fns,
            &mut cache,
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
            &test_fns,
            engines,
            &mut cache,
        ),
        ty::TyProgramKind::Library { .. } => compile::compile_library(
            engines,
            &mut ctx,
            namespace,
            &logged_types,
            &messages_types,
            panic_occurrences,
            &test_fns,
            &mut cache,
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
