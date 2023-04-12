use std::ops::{BitAnd, BitOr, BitXor};

use crate::{
    asm_generation::from_ir::ir_type_size_in_bytes,
    decl_engine::DeclEngine,
    engine_threading::*,
    language::{
        ty::{self, TyIntrinsicFunctionKind},
        CallPath,
    },
    metadata::MetadataManager,
    semantic_analysis::*,
    TypeEngine,
};

use super::{
    convert::{convert_literal_to_constant, convert_resolved_typeid},
    function::FnCompiler,
    types::*,
};

use sway_error::error::CompileError;
use sway_ir::{
    constant::{Constant, ConstantValue},
    context::Context,
    metadata::combine as md_combine,
    module::Module,
    value::Value,
    Instruction, Type,
};
use sway_types::{ident::Ident, span::Spanned};
use sway_utils::mapped_stack::MappedStack;

pub(crate) struct LookupEnv<'a> {
    pub(crate) type_engine: &'a TypeEngine,
    pub(crate) decl_engine: &'a DeclEngine,
    pub(crate) context: &'a mut Context,
    pub(crate) md_mgr: &'a mut MetadataManager,
    pub(crate) module: Module,
    pub(crate) module_ns: Option<&'a namespace::Module>,
    pub(crate) function_compiler: Option<&'a FnCompiler<'a>>,
    pub(crate) lookup: fn(&mut LookupEnv, &CallPath) -> Result<Option<Value>, CompileError>,
}

pub(crate) fn compile_const_decl(
    env: &mut LookupEnv,
    call_path: &CallPath,
) -> Result<Option<Value>, CompileError> {
    // Check if it's a processed local constant.
    if let Some(fn_compiler) = env.function_compiler {
        let mut found_local = false;
        if let Some(local_var) =
            fn_compiler.get_function_var(env.context, call_path.suffix.as_str())
        {
            found_local = true;
            if let Some(constant) = local_var.get_initializer(env.context) {
                return Ok(Some(Value::new_constant(env.context, constant.clone())));
            }

            // Check if a constant was stored to a local variable in the current block.
            let mut stored_const_opt: Option<&Constant> = None;
            for ins in fn_compiler.current_block.instruction_iter(env.context) {
                if let Some(Instruction::Store {
                    dst_val_ptr: dst_val,
                    stored_val,
                }) = ins.get_instruction(env.context)
                {
                    if let Some(Instruction::GetLocal(store_dst_var)) =
                        dst_val.get_instruction(env.context)
                    {
                        if &local_var == store_dst_var {
                            stored_const_opt = stored_val.get_constant(env.context);
                        }
                    }
                }
            }
            if let Some(constant) = stored_const_opt {
                return Ok(Some(Value::new_constant(env.context, constant.clone())));
            }
        }

        if let Some(value) = fn_compiler.get_function_arg(env.context, call_path.suffix.as_str()) {
            found_local = true;
            if value.get_constant(env.context).is_some() {
                return Ok(Some(value));
            }
        }

        if found_local {
            return Ok(None);
        }
    }

    // Check if it's a processed global constant.
    match (
        env.module
            .get_global_constant(env.context, &call_path.as_vec_string()),
        env.module
            .get_global_configurable(env.context, &call_path.as_vec_string()),
        env.module_ns,
    ) {
        (Some(const_val), _, _) => Ok(Some(const_val)),
        (_, Some(config_val), _) => Ok(Some(config_val)),
        (None, None, Some(module_ns)) => {
            // See if we it's a global const and whether we can compile it *now*.
            let decl = module_ns.check_symbol(&call_path.suffix)?;
            let decl_name_value = match decl {
                ty::TyDecl::ConstantDecl { decl_id, .. } => {
                    let ty::TyConstantDecl {
                        call_path,
                        value,
                        is_configurable,
                        ..
                    } = env.decl_engine.get_constant(decl_id);
                    Some((call_path, value, is_configurable))
                }
                _otherwise => None,
            };
            if let Some((call_path, Some(value), is_configurable)) = decl_name_value {
                let const_val = compile_constant_expression(
                    Engines::new(env.type_engine, env.decl_engine),
                    env.context,
                    env.md_mgr,
                    env.module,
                    env.module_ns,
                    env.function_compiler,
                    &call_path,
                    &value,
                    is_configurable,
                )?;
                if !is_configurable {
                    env.module.add_global_constant(
                        env.context,
                        call_path.as_vec_string().to_vec(),
                        const_val,
                    );
                } else {
                    env.module.add_global_configurable(
                        env.context,
                        call_path.as_vec_string().to_vec(),
                        const_val,
                    );
                }
                Ok(Some(const_val))
            } else {
                Ok(None)
            }
        }
        _ => Ok(None),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_constant_expression(
    engines: Engines<'_>,
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    module_ns: Option<&namespace::Module>,
    function_compiler: Option<&FnCompiler>,
    call_path: &CallPath,
    const_expr: &ty::TyExpression,
    is_configurable: bool,
) -> Result<Value, CompileError> {
    let span_id_idx = md_mgr.span_to_md(context, &const_expr.span);

    let constant_evaluated = compile_constant_expression_to_constant(
        engines,
        context,
        md_mgr,
        module,
        module_ns,
        function_compiler,
        const_expr,
    )?;
    if !is_configurable {
        Ok(Value::new_constant(context, constant_evaluated).add_metadatum(context, span_id_idx))
    } else {
        let config_const_name =
            md_mgr.config_const_name_to_md(context, &std::rc::Rc::from(call_path.suffix.as_str()));
        let metadata = md_combine(context, &span_id_idx, &config_const_name);
        Ok(Value::new_configurable(context, constant_evaluated).add_metadatum(context, metadata))
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_constant_expression_to_constant(
    engines: Engines<'_>,
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    module_ns: Option<&namespace::Module>,
    function_compiler: Option<&FnCompiler>,
    const_expr: &ty::TyExpression,
) -> Result<Constant, CompileError> {
    let (type_engine, decl_engine) = engines.unwrap();
    let lookup = &mut LookupEnv {
        type_engine,
        decl_engine,
        context,
        md_mgr,
        module,
        module_ns,
        function_compiler,
        lookup: compile_const_decl,
    };

    let err = match &const_expr.expression {
        // Special case functions because the span in `const_expr` is to the inlined function
        // definition, rather than the actual call site.
        ty::TyExpressionVariant::FunctionApplication { call_path, .. } => {
            Err(CompileError::NonConstantDeclValue {
                span: call_path.span(),
            })
        }
        _otherwise => Err(CompileError::NonConstantDeclValue {
            span: const_expr.span.clone(),
        }),
    };
    let mut known_consts = MappedStack::<Ident, Constant>::new();

    const_eval_typed_expr(lookup, &mut known_consts, const_expr)?.map_or(err, Ok)
}

/// Given an environment mapping names to constants,
/// attempt to evaluate a typed expression to a constant.
fn const_eval_typed_expr(
    lookup: &mut LookupEnv,
    known_consts: &mut MappedStack<Ident, Constant>,
    expr: &ty::TyExpression,
) -> Result<Option<Constant>, CompileError> {
    Ok(match &expr.expression {
        ty::TyExpressionVariant::Literal(l) => Some(convert_literal_to_constant(lookup.context, l)),
        ty::TyExpressionVariant::FunctionApplication {
            arguments, fn_ref, ..
        } => {
            let mut actuals_const: Vec<_> = vec![];
            for arg in arguments {
                let (name, sub_expr) = arg;
                let eval_expr_opt = const_eval_typed_expr(lookup, known_consts, sub_expr)?;
                if let Some(sub_const) = eval_expr_opt {
                    actuals_const.push((name, sub_const));
                }
            }

            // If all actual arguments don't evaluate a constant, bail out.
            // TODO: Explore if we could continue here and if it'll be useful.
            if actuals_const.len() < arguments.len() {
                return Ok(None);
            }
            for (name, cval) in actuals_const.into_iter() {
                known_consts.push(name.clone(), cval);
            }

            // TODO: Handle more than one statement in the block.
            let function_decl = lookup.decl_engine.get_function(fn_ref);
            if function_decl.body.contents.len() > 1 {
                return Ok(None);
            }
            let body_contents_opt = function_decl.body.contents.last();
            let res = if let Some(first_expr) = body_contents_opt {
                const_eval_typed_ast_node(lookup, known_consts, first_expr)?
            } else {
                None
            };
            for (name, _) in arguments {
                known_consts.pop(name);
            }
            res
        }
        ty::TyExpressionVariant::ConstantExpression { const_decl, .. } => {
            let call_path = &const_decl.call_path;
            let name = &call_path.suffix;

            match known_consts.get(name) {
                // 1. Check if name/call_path is in known_consts.
                Some(cvs) => Some(cvs.clone()),
                None => {
                    // 2. Check if name is a global constant.
                    (lookup.lookup)(lookup, call_path)
                        .ok()
                        .flatten()
                        .and_then(|v| v.get_constant_or_configurable(lookup.context).cloned())
                }
            }
        }
        ty::TyExpressionVariant::VariableExpression {
            name, call_path, ..
        } => match known_consts.get(name) {
            // 1. Check if name/call_path is in known_consts.
            Some(cvs) => Some(cvs.clone()),
            None => {
                let call_path = match call_path {
                    Some(call_path) => call_path.clone(),
                    None => CallPath::from(name.clone()),
                };
                // 2. Check if name is a global constant.
                (lookup.lookup)(lookup, &call_path)
                    .ok()
                    .flatten()
                    .and_then(|v| v.get_constant(lookup.context).cloned())
            }
        },
        ty::TyExpressionVariant::StructExpression { fields, .. } => {
            let (mut field_typs, mut field_vals): (Vec<_>, Vec<_>) = (vec![], vec![]);
            for field in fields {
                let ty::TyStructExpressionField { name: _, value, .. } = field;
                let eval_expr_opt = const_eval_typed_expr(lookup, known_consts, value)?;
                if let Some(cv) = eval_expr_opt {
                    field_typs.push(value.return_type);
                    field_vals.push(cv);
                }
            }

            if field_vals.len() < fields.len() {
                // We couldn't evaluate all fields to a constant.
                return Ok(None);
            }
            get_struct_for_types(
                lookup.type_engine,
                lookup.decl_engine,
                lookup.context,
                &field_typs,
            )
            .map_or(None, |struct_ty| {
                Some(Constant::new_struct(
                    lookup.context,
                    struct_ty.get_field_types(lookup.context),
                    field_vals,
                ))
            })
        }
        ty::TyExpressionVariant::Tuple { fields } => {
            let (mut field_typs, mut field_vals): (Vec<_>, Vec<_>) = (vec![], vec![]);
            for value in fields {
                let eval_expr_opt = const_eval_typed_expr(lookup, known_consts, value)?;
                if let Some(cv) = eval_expr_opt {
                    field_typs.push(value.return_type);
                    field_vals.push(cv);
                }
            }
            if field_vals.len() < fields.len() {
                // We couldn't evaluate all fields to a constant.
                return Ok(None);
            }
            create_tuple_aggregate(
                lookup.type_engine,
                lookup.decl_engine,
                lookup.context,
                field_typs,
            )
            .map_or(None, |tuple_ty| {
                Some(Constant::new_struct(
                    lookup.context,
                    tuple_ty.get_field_types(lookup.context),
                    field_vals,
                ))
            })
        }
        ty::TyExpressionVariant::Array {
            elem_type,
            contents,
        } => {
            let (mut element_typs, mut element_vals): (Vec<_>, Vec<_>) = (vec![], vec![]);
            for value in contents {
                let eval_expr_opt = const_eval_typed_expr(lookup, known_consts, value)?;
                if let Some(cv) = eval_expr_opt {
                    element_typs.push(value.return_type);
                    element_vals.push(cv);
                }
            }
            if element_vals.len() < contents.len() || element_typs.is_empty() {
                // We couldn't evaluate all fields to a constant or cannot determine element type.
                return Ok(None);
            }
            let elem_type_info = lookup.type_engine.get(*elem_type);
            if !element_typs.iter().all(|tid| {
                lookup.type_engine.get(*tid).eq(
                    &elem_type_info,
                    Engines::new(lookup.type_engine, lookup.decl_engine),
                )
            }) {
                // This shouldn't happen if the type checker did its job.
                return Ok(None);
            }
            create_array_aggregate(
                lookup.type_engine,
                lookup.decl_engine,
                lookup.context,
                *elem_type,
                element_typs.len().try_into().unwrap(),
            )
            .map_or(None, |array_ty| {
                Some(Constant::new_array(
                    lookup.context,
                    array_ty.get_array_elem_type(lookup.context).unwrap(),
                    element_vals,
                ))
            })
        }
        ty::TyExpressionVariant::EnumInstantiation {
            enum_ref,
            tag,
            contents,
            ..
        } => {
            let enum_decl = lookup.decl_engine.get_enum(enum_ref);
            let aggregate = create_tagged_union_type(
                lookup.type_engine,
                lookup.decl_engine,
                lookup.context,
                &enum_decl.variants,
            );
            if let Ok(enum_ty) = aggregate {
                let tag_value = Constant::new_uint(lookup.context, 64, *tag as u64);
                let mut fields: Vec<Constant> = vec![tag_value];
                match contents {
                    None => fields.push(Constant::new_unit(lookup.context)),
                    Some(subexpr) => {
                        let eval_expr = const_eval_typed_expr(lookup, known_consts, subexpr)?;
                        eval_expr.into_iter().for_each(|enum_val| {
                            fields.push(enum_val);
                        })
                    }
                }
                Some(Constant::new_struct(
                    lookup.context,
                    enum_ty.get_field_types(lookup.context),
                    fields,
                ))
            } else {
                None
            }
        }
        ty::TyExpressionVariant::StructFieldAccess {
            prefix,
            field_to_access,
            resolved_type_of_parent,
            ..
        } => match const_eval_typed_expr(lookup, known_consts, prefix)? {
            Some(Constant {
                value: ConstantValue::Struct(fields),
                ..
            }) => {
                let field_kind = ty::ProjectionKind::StructField {
                    name: field_to_access.name.clone(),
                };
                get_struct_name_field_index_and_type(
                    lookup.type_engine,
                    lookup.decl_engine,
                    *resolved_type_of_parent,
                    field_kind,
                )
                .and_then(|(_struct_name, field_idx_and_type_opt)| {
                    field_idx_and_type_opt.map(|(field_idx, _field_type)| field_idx)
                })
                .and_then(|field_idx| fields.get(field_idx as usize).cloned())
            }
            _ => None,
        },
        ty::TyExpressionVariant::TupleElemAccess {
            prefix,
            elem_to_access_num,
            ..
        } => match const_eval_typed_expr(lookup, known_consts, prefix)? {
            Some(Constant {
                value: ConstantValue::Struct(fields),
                ..
            }) => fields.get(*elem_to_access_num).cloned(),
            _ => None,
        },
        ty::TyExpressionVariant::Return(exp) => const_eval_typed_expr(lookup, known_consts, exp)?,
        ty::TyExpressionVariant::MatchExp { desugared, .. } => {
            const_eval_typed_expr(lookup, known_consts, desugared)?
        }
        ty::TyExpressionVariant::IntrinsicFunction(kind) => {
            const_eval_intrinsic(lookup, known_consts, kind)?
        }
        ty::TyExpressionVariant::ArrayIndex { .. }
        | ty::TyExpressionVariant::CodeBlock(_)
        | ty::TyExpressionVariant::Reassignment(_)
        | ty::TyExpressionVariant::StorageReassignment(_)
        | ty::TyExpressionVariant::FunctionParameter
        | ty::TyExpressionVariant::IfExp { .. }
        | ty::TyExpressionVariant::AsmExpression { .. }
        | ty::TyExpressionVariant::LazyOperator { .. }
        | ty::TyExpressionVariant::AbiCast { .. }
        | ty::TyExpressionVariant::StorageAccess(_)
        | ty::TyExpressionVariant::AbiName(_)
        | ty::TyExpressionVariant::EnumTag { .. }
        | ty::TyExpressionVariant::UnsafeDowncast { .. }
        | ty::TyExpressionVariant::Break
        | ty::TyExpressionVariant::Continue
        | ty::TyExpressionVariant::WhileLoop { .. } => None,
    })
}

fn const_eval_intrinsic(
    lookup: &mut LookupEnv,
    known_consts: &mut MappedStack<Ident, Constant>,
    intrinsic: &TyIntrinsicFunctionKind,
) -> Result<Option<Constant>, CompileError> {
    let args = intrinsic
        .arguments
        .iter()
        .filter_map(|arg| const_eval_typed_expr(lookup, known_consts, arg).transpose())
        .collect::<Result<Vec<_>, CompileError>>()?;

    if args.len() != intrinsic.arguments.len() {
        // We couldn't const-eval all arguments.
        return Ok(None);
    }
    match intrinsic.kind {
        sway_ast::Intrinsic::Add
        | sway_ast::Intrinsic::Sub
        | sway_ast::Intrinsic::Mul
        | sway_ast::Intrinsic::Div
        | sway_ast::Intrinsic::And
        | sway_ast::Intrinsic::Or
        | sway_ast::Intrinsic::Xor => {
            let ty = args[0].ty;
            assert!(
                args.len() == 2 && ty.is_uint(lookup.context) && ty.eq(lookup.context, &args[1].ty)
            );
            let (ConstantValue::Uint(arg1), ConstantValue::Uint(ref arg2)) = (&args[0].value, &args[1].value)
            else {
                panic!("Type checker allowed incorrect args to binary op");
            };
            // All arithmetic is done as if it were u64
            let result = match intrinsic.kind {
                sway_ast::Intrinsic::Add => arg1.checked_add(*arg2),
                sway_ast::Intrinsic::Sub => arg1.checked_sub(*arg2),
                sway_ast::Intrinsic::Mul => arg1.checked_mul(*arg2),
                sway_ast::Intrinsic::Div => arg1.checked_div(*arg2),
                sway_ast::Intrinsic::And => Some(arg1.bitand(arg2)),
                sway_ast::Intrinsic::Or => Some(arg1.bitor(*arg2)),
                sway_ast::Intrinsic::Xor => Some(arg1.bitxor(*arg2)),
                _ => unreachable!(),
            };
            match result {
                Some(sum) => Ok(Some(Constant {
                    ty,
                    value: ConstantValue::Uint(sum),
                })),
                None => Ok(None),
            }
        }
        sway_ast::Intrinsic::SizeOfType => {
            let targ = &intrinsic.type_arguments[0];
            let ir_type = convert_resolved_typeid(
                lookup.type_engine,
                lookup.decl_engine,
                lookup.context,
                &targ.type_id,
                &targ.span,
            )?;
            Ok(Some(Constant {
                ty: Type::get_uint64(lookup.context),
                value: ConstantValue::Uint(ir_type_size_in_bytes(lookup.context, &ir_type)),
            }))
        }
        sway_ast::Intrinsic::SizeOfVal => {
            let val = &intrinsic.arguments[0];
            let type_id = val.return_type;
            let ir_type = convert_resolved_typeid(
                lookup.type_engine,
                lookup.decl_engine,
                lookup.context,
                &type_id,
                &val.span,
            )?;
            Ok(Some(Constant {
                ty: Type::get_uint64(lookup.context),
                value: ConstantValue::Uint(ir_type_size_in_bytes(lookup.context, &ir_type)),
            }))
        }
        sway_ast::Intrinsic::Eq => {
            assert!(args.len() == 2);
            Ok(Some(Constant {
                ty: Type::get_bool(lookup.context),
                value: ConstantValue::Bool(args[0].eq(lookup.context, &args[1])),
            }))
        }
        sway_ast::Intrinsic::Gt => {
            let (ConstantValue::Uint(val1), ConstantValue::Uint(val2)) = (&args[0].value, &args[1].value)
                else {
                    unreachable!("Type checker allowed non integer value for GreaterThan")
                };
            Ok(Some(Constant {
                ty: Type::get_bool(lookup.context),
                value: ConstantValue::Bool(val1 > val2),
            }))
        }
        sway_ast::Intrinsic::Lt => {
            let (ConstantValue::Uint(val1), ConstantValue::Uint(val2)) = (&args[0].value, &args[1].value)
                else {
                    unreachable!("Type checker allowed non integer value for LessThan")
                };
            Ok(Some(Constant {
                ty: Type::get_bool(lookup.context),
                value: ConstantValue::Bool(val1 < val2),
            }))
        }
        sway_ast::Intrinsic::AddrOf => Ok(None),
        sway_ast::Intrinsic::PtrAdd => Ok(None),
        sway_ast::Intrinsic::PtrSub => Ok(None),
        sway_ast::Intrinsic::GetStorageKey
        | sway_ast::Intrinsic::IsReferenceType
        | sway_ast::Intrinsic::Gtf
        | sway_ast::Intrinsic::StateClear
        | sway_ast::Intrinsic::StateLoadWord
        | sway_ast::Intrinsic::StateStoreWord
        | sway_ast::Intrinsic::StateLoadQuad
        | sway_ast::Intrinsic::StateStoreQuad
        | sway_ast::Intrinsic::Log
        | sway_ast::Intrinsic::Revert
        | sway_ast::Intrinsic::Smo => Ok(None),
    }
}

fn const_eval_typed_ast_node(
    lookup: &mut LookupEnv,
    known_consts: &mut MappedStack<Ident, Constant>,
    expr: &ty::TyAstNode,
) -> Result<Option<Constant>, CompileError> {
    match &expr.content {
        ty::TyAstNodeContent::Declaration(_) => {
            // TODO: add the binding to known_consts (if it's a const) and proceed.
            Ok(None)
        }
        ty::TyAstNodeContent::Expression(e) | ty::TyAstNodeContent::ImplicitReturnExpression(e) => {
            const_eval_typed_expr(lookup, known_consts, e)
        }
        ty::TyAstNodeContent::SideEffect(_) => Ok(None),
    }
}
