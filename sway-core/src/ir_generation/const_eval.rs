use std::ops::{BitAnd, BitOr, BitXor};

use crate::{
    asm_generation::from_ir::{ir_type_size_in_bytes, ir_type_str_size_in_bytes},
    engine_threading::*,
    language::{
        ty::{self, TyConstantDecl, TyIntrinsicFunctionKind},
        CallPath,
    },
    metadata::MetadataManager,
    semantic_analysis::*,
    UnifyCheck,
};

use super::{
    convert::{convert_literal_to_constant, convert_resolved_typeid},
    function::FnCompiler,
    types::*,
};

use sway_ast::Intrinsic;
use sway_error::error::CompileError;
use sway_ir::{
    constant::{Constant, ConstantValue},
    context::Context,
    metadata::combine as md_combine,
    module::Module,
    value::Value,
    Instruction, Type,
};
use sway_types::{ident::Ident, span::Spanned, Span};
use sway_utils::mapped_stack::MappedStack;

enum ConstEvalError {
    CompileError(CompileError),
    CannotBeEvaluatedToConst {
        // This is not used at the moment because we do not give detailed description of why a
        // const eval failed.
        // Nonetheless, this is used in tests to help debug.
        #[allow(dead_code)]
        span: Span,
    },
}

pub(crate) struct LookupEnv<'a, 'eng> {
    pub(crate) engines: &'a Engines,
    pub(crate) context: &'a mut Context<'eng>,
    pub(crate) md_mgr: &'a mut MetadataManager,
    pub(crate) module: Module,
    pub(crate) module_ns: Option<&'a namespace::Module>,
    pub(crate) function_compiler: Option<&'a FnCompiler<'a>>,
    #[allow(clippy::type_complexity)]
    pub(crate) lookup: fn(
        &mut LookupEnv,
        &CallPath,
        &Option<TyConstantDecl>,
    ) -> Result<Option<Value>, CompileError>,
}

pub(crate) fn compile_const_decl(
    env: &mut LookupEnv,
    call_path: &CallPath,
    const_decl: &Option<TyConstantDecl>,
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
            let decl = module_ns.check_symbol(&call_path.suffix);
            let const_decl = match const_decl {
                Some(decl) => Some(decl),
                None => None,
            };
            let const_decl = match decl {
                Ok(decl) => match decl {
                    ty::TyDecl::ConstantDecl(ty::ConstantDecl { decl_id, .. }) => {
                        Some(env.engines.de().get_constant(decl_id))
                    }
                    _otherwise => const_decl.cloned(),
                },
                Err(_) => const_decl.cloned(),
            };
            match const_decl {
                Some(const_decl) => {
                    let ty::TyConstantDecl {
                        call_path,
                        value,
                        is_configurable,
                        ..
                    } = const_decl;
                    if value.is_none() {
                        return Ok(None);
                    }

                    let const_val = compile_constant_expression(
                        env.engines,
                        env.context,
                        env.md_mgr,
                        env.module,
                        env.module_ns,
                        env.function_compiler,
                        &call_path,
                        &value.unwrap(),
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
                }
                None => Ok(None),
            }
        }
        _ => Ok(None),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_constant_expression(
    engines: &Engines,
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
    engines: &Engines,
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    module_ns: Option<&namespace::Module>,
    function_compiler: Option<&FnCompiler>,
    const_expr: &ty::TyExpression,
) -> Result<Constant, CompileError> {
    let lookup = &mut LookupEnv {
        engines,
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

    match const_eval_typed_expr(lookup, &mut known_consts, const_expr) {
        Ok(Some(constant)) => Ok(constant),
        _ => err,
    }
}

/// Given an environment mapping names to constants,
/// attempt to evaluate a typed expression to a constant.
fn const_eval_typed_expr(
    lookup: &mut LookupEnv,
    known_consts: &mut MappedStack<Ident, Constant>,
    expr: &ty::TyExpression,
) -> Result<Option<Constant>, ConstEvalError> {
    Ok(match &expr.expression {
        ty::TyExpressionVariant::Literal(l) => Some(convert_literal_to_constant(lookup.context, l)),
        ty::TyExpressionVariant::FunctionApplication {
            arguments,
            fn_ref,
            call_path,
            ..
        } => {
            let mut actuals_const: Vec<_> = vec![];

            for arg in arguments {
                let (name, sub_expr) = arg;
                let eval_expr_opt = const_eval_typed_expr(lookup, known_consts, sub_expr)?;
                if let Some(sub_const) = eval_expr_opt {
                    actuals_const.push((name, sub_const));
                } else {
                    // If all actual arguments don't evaluate a constant, bail out.
                    // TODO: Explore if we could continue here and if it'll be useful.
                    return Err(ConstEvalError::CannotBeEvaluatedToConst {
                        span: call_path.span(),
                    });
                }
            }

            assert!(actuals_const.len() == arguments.len());

            for (name, cval) in actuals_const.into_iter() {
                known_consts.push(name.clone(), cval);
            }

            let function_decl = lookup.engines.de().get_function(fn_ref);
            let res = const_eval_codeblock(lookup, known_consts, &function_decl.body);

            for (name, _) in arguments {
                known_consts.pop(name);
            }

            res?
        }
        ty::TyExpressionVariant::ConstantExpression { const_decl, .. } => {
            let call_path = &const_decl.call_path;
            let name = &call_path.suffix;

            match known_consts.get(name) {
                // 1. Check if name/call_path is in known_consts.
                Some(cvs) => Some(cvs.clone()),
                None => {
                    // 2. Check if name is a global constant.
                    (lookup.lookup)(lookup, call_path, &Some(*const_decl.clone()))
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
                (lookup.lookup)(lookup, &call_path, &None)
                    .ok()
                    .flatten()
                    .and_then(|v| v.get_constant(lookup.context).cloned())
            }
        },
        ty::TyExpressionVariant::StructExpression {
            fields,
            instantiation_span,
            ..
        } => {
            let (mut field_typs, mut field_vals): (Vec<_>, Vec<_>) = (vec![], vec![]);

            for field in fields {
                let ty::TyStructExpressionField { name: _, value, .. } = field;
                let eval_expr_opt = const_eval_typed_expr(lookup, known_consts, value)?;
                if let Some(cv) = eval_expr_opt {
                    field_typs.push(value.return_type);
                    field_vals.push(cv);
                } else {
                    return Err(ConstEvalError::CannotBeEvaluatedToConst {
                        span: instantiation_span.clone(),
                    });
                }
            }

            assert!(field_typs.len() == fields.len());
            assert!(field_vals.len() == fields.len());

            get_struct_for_types(
                lookup.engines.te(),
                lookup.engines.de(),
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
                } else {
                    return Err(ConstEvalError::CannotBeEvaluatedToConst {
                        span: expr.span.clone(),
                    });
                }
            }

            assert!(field_typs.len() == fields.len());
            assert!(field_vals.len() == fields.len());

            create_tuple_aggregate(
                lookup.engines.te(),
                lookup.engines.de(),
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
                } else {
                    return Err(ConstEvalError::CannotBeEvaluatedToConst {
                        span: expr.span.clone(),
                    });
                }
            }

            assert!(element_typs.len() == contents.len());
            assert!(element_vals.len() == contents.len());

            let te = lookup.engines.te();

            assert!({
                let unify_check = UnifyCheck::coercion(lookup.engines);
                element_typs
                    .iter()
                    .all(|tid| unify_check.check(*tid, *elem_type))
            });

            create_array_aggregate(
                te,
                lookup.engines.de(),
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
            variant_instantiation_span,
            ..
        } => {
            let enum_decl = lookup.engines.de().get_enum(enum_ref);
            let aggregate = create_tagged_union_type(
                lookup.engines.te(),
                lookup.engines.de(),
                lookup.context,
                &enum_decl.variants,
            );

            if let Ok(enum_ty) = aggregate {
                let tag_value = Constant::new_uint(lookup.context, 64, *tag as u64);
                let mut fields: Vec<Constant> = vec![tag_value];

                match contents {
                    None => fields.push(Constant::new_unit(lookup.context)),
                    Some(subexpr) => match const_eval_typed_expr(lookup, known_consts, subexpr)? {
                        Some(constant) => fields.push(constant),
                        None => {
                            return Err(ConstEvalError::CannotBeEvaluatedToConst {
                                span: variant_instantiation_span.clone(),
                            })
                        }
                    },
                }

                let fields_tys = enum_ty.get_field_types(lookup.context);
                Some(Constant::new_struct(lookup.context, fields_tys, fields))
            } else {
                return Err(ConstEvalError::CannotBeEvaluatedToConst {
                    span: expr.span.clone(),
                });
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
                    lookup.engines.te(),
                    lookup.engines.de(),
                    *resolved_type_of_parent,
                    field_kind,
                )
                .and_then(|(_struct_name, field_idx_and_type_opt)| {
                    field_idx_and_type_opt.map(|(field_idx, _field_type)| field_idx)
                })
                .and_then(|field_idx| fields.get(field_idx as usize).cloned())
            }
            _ => {
                return Err(ConstEvalError::CannotBeEvaluatedToConst {
                    span: expr.span.clone(),
                })
            }
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
            _ => {
                return Err(ConstEvalError::CannotBeEvaluatedToConst {
                    span: expr.span.clone(),
                })
            }
        },
        // we could allow non-local control flow in pure functions, but it would
        // require some more work and at this point it's not clear if it is too useful
        // for constant initializers -- the user can always refactor their pure functions
        // to not use the return statement
        ty::TyExpressionVariant::Return(exp) => {
            return Err(ConstEvalError::CannotBeEvaluatedToConst {
                span: exp.span.clone(),
            })
        }
        ty::TyExpressionVariant::MatchExp { desugared, .. } => {
            const_eval_typed_expr(lookup, known_consts, desugared)?
        }
        ty::TyExpressionVariant::IntrinsicFunction(kind) => {
            const_eval_intrinsic(lookup, known_consts, kind)?
        }
        ty::TyExpressionVariant::IfExp {
            condition,
            then,
            r#else,
        } => {
            match const_eval_typed_expr(lookup, known_consts, condition)? {
                Some(Constant {
                    value: ConstantValue::Bool(cond),
                    ..
                }) => {
                    if cond {
                        const_eval_typed_expr(lookup, known_consts, then)?
                    } else if let Some(r#else) = r#else {
                        const_eval_typed_expr(lookup, known_consts, r#else)?
                    } else {
                        // missing 'else' branch:
                        // we probably don't really care about evaluating
                        // const expressions of the unit type
                        None
                    }
                }
                _ => {
                    return Err(ConstEvalError::CannotBeEvaluatedToConst {
                        span: expr.span.clone(),
                    })
                }
            }
        }
        ty::TyExpressionVariant::CodeBlock(codeblock) => {
            const_eval_codeblock(lookup, known_consts, codeblock)?
        }
        ty::TyExpressionVariant::ArrayIndex { prefix, index } => {
            let prefix = const_eval_typed_expr(lookup, known_consts, prefix)?;
            let index = const_eval_typed_expr(lookup, known_consts, index)?;
            match (prefix, index) {
                (
                    Some(Constant {
                        value: ConstantValue::Array(items),
                        ..
                    }),
                    Some(Constant {
                        value: ConstantValue::Uint(index),
                        ..
                    }),
                ) => {
                    let count = items.len() as u64;
                    if index < count {
                        Some(items[index as usize].clone())
                    } else {
                        return Err(ConstEvalError::CompileError(
                            CompileError::ArrayOutOfBounds {
                                index,
                                count,
                                span: expr.span.clone(),
                            },
                        ));
                    }
                }
                _ => {
                    return Err(ConstEvalError::CannotBeEvaluatedToConst {
                        span: expr.span.clone(),
                    })
                }
            }
        }
        ty::TyExpressionVariant::Reassignment(_)
        | ty::TyExpressionVariant::FunctionParameter
        | ty::TyExpressionVariant::AsmExpression { .. }
        | ty::TyExpressionVariant::LazyOperator { .. }
        | ty::TyExpressionVariant::AbiCast { .. }
        | ty::TyExpressionVariant::StorageAccess(_)
        | ty::TyExpressionVariant::AbiName(_)
        | ty::TyExpressionVariant::EnumTag { .. }
        | ty::TyExpressionVariant::UnsafeDowncast { .. }
        | ty::TyExpressionVariant::Break
        | ty::TyExpressionVariant::Continue
        | ty::TyExpressionVariant::WhileLoop { .. } => {
            return Err(ConstEvalError::CannotBeEvaluatedToConst {
                span: expr.span.clone(),
            })
        }
    })
}

// the (constant) value of a codeblock is essentially it's last expression if there is one
// or if it makes sense as the last expression, e.g. a dangling let-expression in a codeblock
// would be an evaluation error
fn const_eval_codeblock(
    lookup: &mut LookupEnv,
    known_consts: &mut MappedStack<Ident, Constant>,
    codeblock: &ty::TyCodeBlock,
) -> Result<Option<Constant>, ConstEvalError> {
    // the current result
    let mut result: Result<Option<Constant>, ConstEvalError> = Ok(None);
    // keep track of new bindings for this codeblock
    let mut bindings: Vec<_> = vec![];

    for ast_node in &codeblock.contents {
        result = match &ast_node.content {
            ty::TyAstNodeContent::Declaration(decl @ ty::TyDecl::VariableDecl(var_decl)) => {
                if let Ok(Some(rhs)) = const_eval_typed_expr(lookup, known_consts, &var_decl.body) {
                    known_consts.push(var_decl.name.clone(), rhs);
                    bindings.push(var_decl.name.clone());
                    Ok(None)
                } else {
                    Err(ConstEvalError::CannotBeEvaluatedToConst {
                        span: decl.span().clone(),
                    })
                }
            }
            ty::TyAstNodeContent::Declaration(ty::TyDecl::ConstantDecl(const_decl)) => {
                let ty_const_decl = lookup.engines.de().get_constant(&const_decl.decl_id);
                if let Some(constant) = ty_const_decl
                    .value
                    .and_then(|expr| const_eval_typed_expr(lookup, known_consts, &expr).ok())
                    .flatten()
                {
                    known_consts.push(const_decl.name.clone(), constant);
                    bindings.push(const_decl.name.clone());
                    Ok(None)
                } else {
                    Err(ConstEvalError::CannotBeEvaluatedToConst {
                        span: const_decl.decl_span.clone(),
                    })
                }
            }
            ty::TyAstNodeContent::Declaration(_) => Ok(None),
            ty::TyAstNodeContent::Expression(e) => {
                if const_eval_typed_expr(lookup, known_consts, e).is_err() {
                    Err(ConstEvalError::CannotBeEvaluatedToConst {
                        span: e.span.clone(),
                    })
                } else {
                    Ok(None)
                }
            }
            ty::TyAstNodeContent::ImplicitReturnExpression(e) => {
                if let Ok(Some(constant)) = const_eval_typed_expr(lookup, known_consts, e) {
                    Ok(Some(constant))
                } else {
                    Err(ConstEvalError::CannotBeEvaluatedToConst {
                        span: e.span.clone(),
                    })
                }
            }
            ty::TyAstNodeContent::SideEffect(_) => Err(ConstEvalError::CannotBeEvaluatedToConst {
                span: ast_node.span.clone(),
            }),
        };

        if result.is_err() {
            break;
        }
    }

    // remove introduced vars/consts from scope at the end of the codeblock
    for name in bindings {
        known_consts.pop(&name)
    }

    result
}

fn const_eval_intrinsic(
    lookup: &mut LookupEnv,
    known_consts: &mut MappedStack<Ident, Constant>,
    intrinsic: &TyIntrinsicFunctionKind,
) -> Result<Option<Constant>, ConstEvalError> {
    let mut args = vec![];
    for arg in intrinsic.arguments.iter() {
        if let Ok(Some(constant)) = const_eval_typed_expr(lookup, known_consts, arg) {
            args.push(constant);
        } else {
            return Err(ConstEvalError::CannotBeEvaluatedToConst {
                span: arg.span.clone(),
            });
        }
    }

    assert!(args.len() == intrinsic.arguments.len());

    match intrinsic.kind {
        sway_ast::Intrinsic::Add
        | sway_ast::Intrinsic::Sub
        | sway_ast::Intrinsic::Mul
        | sway_ast::Intrinsic::Div
        | sway_ast::Intrinsic::And
        | sway_ast::Intrinsic::Or
        | sway_ast::Intrinsic::Xor
        | sway_ast::Intrinsic::Mod => {
            let ty = args[0].ty;
            assert!(
                args.len() == 2 && ty.is_uint(lookup.context) && ty.eq(lookup.context, &args[1].ty)
            );
            let (ConstantValue::Uint(arg1), ConstantValue::Uint(ref arg2)) =
                (&args[0].value, &args[1].value)
            else {
                panic!("Type checker allowed incorrect args to binary op");
            };

            // All arithmetic is done as if it were u64
            let result = match intrinsic.kind {
                Intrinsic::Add => arg1.checked_add(*arg2),
                Intrinsic::Sub => arg1.checked_sub(*arg2),
                Intrinsic::Mul => arg1.checked_mul(*arg2),
                Intrinsic::Div => arg1.checked_div(*arg2),
                Intrinsic::And => Some(arg1.bitand(arg2)),
                Intrinsic::Or => Some(arg1.bitor(*arg2)),
                Intrinsic::Xor => Some(arg1.bitxor(*arg2)),
                Intrinsic::Mod => arg1.checked_rem(*arg2),
                _ => unreachable!(),
            };

            match result {
                Some(sum) => Ok(Some(Constant {
                    ty,
                    value: ConstantValue::Uint(sum),
                })),
                None => Err(ConstEvalError::CannotBeEvaluatedToConst {
                    span: intrinsic.span.clone(),
                }),
            }
        }
        sway_ast::Intrinsic::Lsh | sway_ast::Intrinsic::Rsh => {
            let ty = args[0].ty;
            assert!(
                args.len() == 2
                    && ty.is_uint(lookup.context)
                    && args[1].ty.is_uint64(lookup.context)
            );

            let (ConstantValue::Uint(arg1), ConstantValue::Uint(ref arg2)) =
                (&args[0].value, &args[1].value)
            else {
                panic!("Type checker allowed incorrect args to binary op");
            };

            let result = match intrinsic.kind {
                Intrinsic::Lsh => u32::try_from(*arg2)
                    .ok()
                    .and_then(|arg2| arg1.checked_shl(arg2)),
                Intrinsic::Rsh => u32::try_from(*arg2)
                    .ok()
                    .and_then(|arg2| arg1.checked_shr(arg2)),
                _ => unreachable!(),
            };

            match result {
                Some(sum) => Ok(Some(Constant {
                    ty,
                    value: ConstantValue::Uint(sum),
                })),
                None => Err(ConstEvalError::CannotBeEvaluatedToConst {
                    span: intrinsic.span.clone(),
                }),
            }
        }
        sway_ast::Intrinsic::SizeOfType => {
            let targ = &intrinsic.type_arguments[0];
            let ir_type = convert_resolved_typeid(
                lookup.engines.te(),
                lookup.engines.de(),
                lookup.context,
                &targ.type_id,
                &targ.span,
            )
            .map_err(ConstEvalError::CompileError)?;
            Ok(Some(Constant {
                ty: Type::get_uint64(lookup.context),
                value: ConstantValue::Uint(ir_type_size_in_bytes(lookup.context, &ir_type)),
            }))
        }
        sway_ast::Intrinsic::SizeOfVal => {
            let val = &intrinsic.arguments[0];
            let type_id = val.return_type;
            let ir_type = convert_resolved_typeid(
                lookup.engines.te(),
                lookup.engines.de(),
                lookup.context,
                &type_id,
                &val.span,
            )
            .map_err(ConstEvalError::CompileError)?;
            Ok(Some(Constant {
                ty: Type::get_uint64(lookup.context),
                value: ConstantValue::Uint(ir_type_size_in_bytes(lookup.context, &ir_type)),
            }))
        }
        sway_ast::Intrinsic::SizeOfStr => {
            let targ = &intrinsic.type_arguments[0];
            let ir_type = convert_resolved_typeid(
                lookup.engines.te(),
                lookup.engines.de(),
                lookup.context,
                &targ.type_id,
                &targ.span,
            )
            .map_err(ConstEvalError::CompileError)?;
            Ok(Some(Constant {
                ty: Type::get_uint64(lookup.context),
                value: ConstantValue::Uint(ir_type_str_size_in_bytes(lookup.context, &ir_type)),
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
            let (ConstantValue::Uint(val1), ConstantValue::Uint(val2)) =
                (&args[0].value, &args[1].value)
            else {
                unreachable!("Type checker allowed non integer value for GreaterThan")
            };
            Ok(Some(Constant {
                ty: Type::get_bool(lookup.context),
                value: ConstantValue::Bool(val1 > val2),
            }))
        }
        sway_ast::Intrinsic::Lt => {
            let (ConstantValue::Uint(val1), ConstantValue::Uint(val2)) =
                (&args[0].value, &args[1].value)
            else {
                unreachable!("Type checker allowed non integer value for LessThan")
            };
            Ok(Some(Constant {
                ty: Type::get_bool(lookup.context),
                value: ConstantValue::Bool(val1 < val2),
            }))
        }
        sway_ast::Intrinsic::AddrOf
        | sway_ast::Intrinsic::PtrAdd
        | sway_ast::Intrinsic::PtrSub
        | sway_ast::Intrinsic::IsReferenceType
        | sway_ast::Intrinsic::IsStrType
        | sway_ast::Intrinsic::Gtf
        | sway_ast::Intrinsic::StateClear
        | sway_ast::Intrinsic::StateLoadWord
        | sway_ast::Intrinsic::StateStoreWord
        | sway_ast::Intrinsic::StateLoadQuad
        | sway_ast::Intrinsic::StateStoreQuad
        | sway_ast::Intrinsic::Log
        | sway_ast::Intrinsic::Revert
        | sway_ast::Intrinsic::Smo => Err(ConstEvalError::CannotBeEvaluatedToConst {
            span: intrinsic.span.clone(),
        }),
        sway_ast::Intrinsic::Not => {
            // Not works only with uint at the moment
            // `bool` ops::Not implementation uses `__eq`.

            assert!(args.len() == 1 && args[0].ty.is_uint(lookup.context));

            let Some(arg) = args.into_iter().next() else {
                unreachable!("Unexpected 'not' without any arguments");
            };

            let ConstantValue::Uint(v) = arg.value else {
                unreachable!("Type checker allowed non integer value for Not");
            };

            let v = match arg.ty.get_uint_width(lookup.context) {
                Some(8) => !(v as u8) as u64,
                Some(16) => !(v as u16) as u64,
                Some(32) => !(v as u32) as u64,
                Some(64) => !v,
                _ => unreachable!("Invalid unsigned integer width"),
            };

            Ok(Some(Constant {
                ty: arg.ty,
                value: ConstantValue::Uint(v),
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sway_error::handler::Handler;
    use sway_ir::Kind;

    /// This function validates if an expression can be converted to [Constant].
    ///
    /// The flag `is_constant` is used to define if the expression should be convertible or not.
    /// `prefix` is any valid code at top level, useful to declare types.
    ///
    /// Example:
    ///
    /// ```rust,ignore
    /// assert_is_constant(true, "enum Color { Blue: u64 }", "Color::Blue(1)");
    /// assert_is_constant(false, "", "{return 1; 1}");
    /// ```
    ///
    /// It DOES NOT have access to the std lib, and constants, and other features that demand full compilation.
    fn assert_is_constant(is_constant: bool, prefix: &str, expr: &str) {
        let engines = Engines::default();
        let handler = Handler::default();
        let mut context = Context::new(engines.se());
        let mut md_mgr = MetadataManager::default();
        let core_lib = namespace::Module::default();

        let mut performance_data = sway_utils::PerformanceData::default();

        let r = crate::compile_to_ast(
            &handler,
            &engines,
            std::sync::Arc::from(format!("library; {prefix} fn f() -> u64 {{ {expr}; 0 }}")),
            core_lib,
            None,
            "test",
            &mut performance_data,
        );

        let (errors, _warnings) = handler.consume();

        if !errors.is_empty() {
            panic!("{:#?}", errors);
        }

        let f = r.unwrap();
        let f = f.typed.unwrap();

        let f = f
            .declarations
            .iter()
            .find_map(|x| match x {
                ty::TyDecl::FunctionDecl(x) if x.name.as_str() == "f" => Some(x),
                _ => None,
            })
            .expect("An function named `f` was not found.");

        let f = engines.de().get_function(&f.decl_id);
        let expr_under_test = f.body.contents.first().unwrap();

        let expr_under_test = match &expr_under_test.content {
            ty::TyAstNodeContent::Expression(expr_under_test) => expr_under_test.clone(),
            ty::TyAstNodeContent::Declaration(crate::language::ty::TyDecl::ConstantDecl(decl)) => {
                let decl = engines.de().get_constant(&decl.decl_id);
                decl.value.unwrap()
            }
            x => todo!("{x:?}"),
        };

        let module = Module::new(&mut context, Kind::Library);
        let actual_constant = compile_constant_expression_to_constant(
            &engines,
            &mut context,
            &mut md_mgr,
            module,
            None,
            None,
            &expr_under_test,
        );

        match (is_constant, actual_constant) {
            (true, Ok(_)) => {}
            (true, Err(err)) => {
                panic!("Expression cannot be converted to constant: {expr:?}\nPrefix: {prefix:?}\nExpr:{expr_under_test:#?}\nError: {err:#?}");
            }
            (false, Ok(constant)) => {
                panic!("Expression unexpectedly can be converted to constant: {expr:?}\nPrefix: {prefix:?}\nExpr:{expr_under_test:#?}\nConstant: {constant:#?}");
            }
            (false, Err(_)) => {}
        }
    }

    #[test]
    fn const_eval_test() {
        // Expressions that can be converted to constant
        assert_is_constant(true, "", "1");
        assert_is_constant(true, "", "true");
        assert_is_constant(true, "fn one() -> u64 { 1 }", "one()");
        assert_is_constant(true, "fn id(x: u64) -> u64 { x }", "id(1)");
        assert_is_constant(true, "enum Color { Blue: () }", "Color::Blue");
        assert_is_constant(true, "enum Color { Blue: u64 }", "Color::Blue(1)");
        assert_is_constant(true, "struct Person { age: u64 }", "Person { age: 1 }");
        assert_is_constant(true, "struct Person { age: u64 }", "Person { age: 1 }.age");
        assert_is_constant(true, "", "if true { 1 } else { 0 }");
        assert_is_constant(true, "", "(0,1).0");
        assert_is_constant(true, "", "[0,1][0]");

        // Expressions that cannot be converted to constant
        assert_is_constant(false, "", "{ return 1; }");
        assert_is_constant(false, "", "{ return 1; 1}");
        assert_is_constant(
            false,
            "enum Color { Blue: u64 }",
            "Color::Blue({ return 1; 1})",
        );
        assert_is_constant(
            false,
            "struct Person { age: u64 }",
            "Person { age: { return 1; 1} }",
        );
        assert_is_constant(
            false,
            "struct Person { age: u64 }",
            "Person { age: { let mut x = 0; x = 1; 1} }",
        );
        // At the moment this is not constant because of the "return"
        assert_is_constant(false, "fn id(x: u64) -> u64 { return x; }", "id(1)");
        assert_is_constant(false, "", "[0,1][2]");
        assert_is_constant(
            false,
            "enum Color { Blue: u64 }",
            "Color::Blue({return 1;})",
        );

        // Code blocks that can be converted to constants
        assert_is_constant(true, "", "{ 1 }");
        assert_is_constant(true, "", "{ let a = 1; a }");
        assert_is_constant(true, "", "{ const a = 1; a }");
        assert_is_constant(true, "", "{ struct A {} 1 }");
        assert_is_constant(true, "fn id(x: u64) -> u64 { { let x = 2; }; x }", "id(1)");

        // Code blocks that cannot be converted to constants
        assert_is_constant(false, "", "{ let a = 1; }");
        assert_is_constant(false, "", "{ const a = 1; }");
        assert_is_constant(false, "", "{ struct A {} }");
        assert_is_constant(false, "", "{ return 1; 1 }");
        assert_is_constant(false, "", "{ }");
        assert_is_constant(false, "fn id(x: u64) -> u64 { { return 1; }; x }", "id(1)");
    }
}
