use std::ops::{BitAnd, BitOr, BitXor, Not, Rem};

use crate::{
    engine_threading::*,
    language::{
        ty::{self, TyConstantDecl, TyIntrinsicFunctionKind},
        CallPath, Literal,
    },
    metadata::MetadataManager,
    semantic_analysis::*,
    AbiEncodeSizeHint, TypeInfo, UnifyCheck,
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
    InstOp, Instruction, Type, TypeContent,
};
use sway_types::{ident::Ident, integer_bits::IntegerBits, span::Spanned, Named, Span};
use sway_utils::mapped_stack::MappedStack;

#[derive(Debug)]
enum ConstEvalError {
    CompileError,
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
                if let Some(Instruction {
                    op:
                        InstOp::Store {
                            dst_val_ptr: dst_val,
                            stored_val,
                        },
                    ..
                }) = ins.get_instruction(env.context)
                {
                    if let Some(Instruction {
                        op: InstOp::GetLocal(store_dst_var),
                        ..
                    }) = dst_val.get_instruction(env.context)
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
            let decl = module_ns.current_items().check_symbol(&call_path.suffix);
            let const_decl = match const_decl {
                Some(decl) => Some(decl),
                None => None,
            };
            let const_decl = match decl {
                Ok(decl) => match decl.expect_typed() {
                    ty::TyDecl::ConstantDecl(ty::ConstantDecl { decl_id, .. }) => {
                        Some((*env.engines.de().get_constant(&decl_id)).clone())
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

                    let Some(value) = value else {
                        return Ok(None);
                    };

                    let const_val = compile_constant_expression(
                        env.engines,
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
                        let const_val = if env.context.experimental.new_encoding {
                            // This expression must be `encode(something)`.
                            // We want the something here
                            let value_type = match &value.expression {
                                ty::TyExpressionVariant::FunctionApplication {
                                    arguments, ..
                                } => arguments[0].1.return_type,
                                _ => unreachable!("non encoded configurable inside encoding v1"),
                            };

                            let abi_encode_size_hint = env
                                .engines
                                .te()
                                .get(value_type)
                                .abi_encode_size_hint(env.engines);

                            let buffer_size = match abi_encode_size_hint {
                                AbiEncodeSizeHint::CustomImpl
                                | AbiEncodeSizeHint::PotentiallyInfinite => {
                                    return Err(
                                        CompileError::CannotBeEvaluatedToConfigurableSizeUnknown {
                                            span: value.span.clone(),
                                        },
                                    );
                                }
                                AbiEncodeSizeHint::Exact(len) => len,
                                AbiEncodeSizeHint::Range(_, max) => max,
                            };

                            let mut bytes =
                                match &const_val.get_configurable(env.context).unwrap().value {
                                    ConstantValue::RawUntypedSlice(bytes) => Some(bytes.clone()),
                                    _ => {
                                        return Err(CompileError::NonConstantDeclValue {
                                            span: call_path.span(),
                                        });
                                    }
                                }
                                .unwrap();

                            // Pad the encoded buffer if needed
                            assert!(
                                buffer_size >= bytes.len(),
                                "assert {} >= {}",
                                buffer_size,
                                bytes.len()
                            );
                            let padding_size = buffer_size.saturating_sub(bytes.len());
                            bytes.extend((0..padding_size).map(|_| 0));

                            let elements = bytes
                                .iter()
                                .map(|x| Constant::new_uint(env.context, 8, *x as u64))
                                .collect();
                            let array = Constant::new_array(
                                env.context,
                                Type::get_uint8(env.context),
                                elements,
                            );
                            let md_idx = const_val.get_metadata(env.context);
                            Value::new_configurable(env.context, array)
                                .add_metadatum(env.context, md_idx)
                        } else {
                            const_val
                        };

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
        is_configurable,
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
    allow_configurables: bool,
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
            let span = call_path.span();
            let span = if span == Span::dummy() {
                const_expr.span.clone()
            } else {
                span
            };
            Err(CompileError::NonConstantDeclValue { span })
        }
        _otherwise => Err(CompileError::NonConstantDeclValue {
            span: const_expr.span.clone(),
        }),
    };
    let mut known_consts = MappedStack::<Ident, Constant>::new();

    match const_eval_typed_expr(lookup, &mut known_consts, const_expr, allow_configurables) {
        Ok(Some(constant)) => Ok(constant),
        Ok(None) => err,
        Err(_) => err,
    }
}

/// Given an environment mapping names to constants,
/// attempt to evaluate a typed expression to a constant.
fn const_eval_typed_expr(
    lookup: &mut LookupEnv,
    known_consts: &mut MappedStack<Ident, Constant>,
    expr: &ty::TyExpression,
    allow_configurables: bool,
) -> Result<Option<Constant>, ConstEvalError> {
    if let TypeInfo::ErrorRecovery(_) = &*lookup.engines.te().get(expr.return_type) {
        return Err(ConstEvalError::CannotBeEvaluatedToConst {
            span: expr.span.clone(),
        });
    }

    Ok(match &expr.expression {
        ty::TyExpressionVariant::Literal(Literal::Numeric(n)) => {
            let implied_lit = match &*lookup.engines.te().get(expr.return_type) {
                TypeInfo::UnsignedInteger(IntegerBits::Eight) => Literal::U8(*n as u8),
                _ => Literal::U64(*n),
            };
            Some(convert_literal_to_constant(lookup.context, &implied_lit))
        }
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
                let eval_expr_opt =
                    const_eval_typed_expr(lookup, known_consts, sub_expr, allow_configurables)?;
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
            let res = const_eval_codeblock(
                lookup,
                known_consts,
                &function_decl.body,
                allow_configurables,
            );

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
                        .and_then(|v| {
                            if allow_configurables {
                                v.get_constant_or_configurable(lookup.context).cloned()
                            } else {
                                v.get_constant(lookup.context).cloned()
                            }
                        })
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
                let eval_expr_opt =
                    const_eval_typed_expr(lookup, known_consts, value, allow_configurables)?;
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
                let eval_expr_opt =
                    const_eval_typed_expr(lookup, known_consts, value, allow_configurables)?;
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
                &field_typs,
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
                let eval_expr_opt =
                    const_eval_typed_expr(lookup, known_consts, value, allow_configurables)?;
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

            let arr = create_array_aggregate(
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
            });

            arr
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
                    Some(subexpr) => match const_eval_typed_expr(
                        lookup,
                        known_consts,
                        subexpr,
                        allow_configurables,
                    )? {
                        Some(constant) => fields.push(constant),
                        None => {
                            return Err(ConstEvalError::CannotBeEvaluatedToConst {
                                span: variant_instantiation_span.clone(),
                            });
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
        } => match const_eval_typed_expr(lookup, known_consts, prefix, allow_configurables)? {
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
                });
            }
        },
        ty::TyExpressionVariant::TupleElemAccess {
            prefix,
            elem_to_access_num,
            ..
        } => match const_eval_typed_expr(lookup, known_consts, prefix, allow_configurables)? {
            Some(Constant {
                value: ConstantValue::Struct(fields),
                ..
            }) => fields.get(*elem_to_access_num).cloned(),
            _ => {
                return Err(ConstEvalError::CannotBeEvaluatedToConst {
                    span: expr.span.clone(),
                });
            }
        },
        ty::TyExpressionVariant::ImplicitReturn(e) => {
            if let Ok(Some(constant)) =
                const_eval_typed_expr(lookup, known_consts, e, allow_configurables)
            {
                Some(constant)
            } else {
                return Err(ConstEvalError::CannotBeEvaluatedToConst {
                    span: e.span.clone(),
                });
            }
        }
        // we could allow non-local control flow in pure functions, but it would
        // require some more work and at this point it's not clear if it is too useful
        // for constant initializers -- the user can always refactor their pure functions
        // to not use the return statement
        ty::TyExpressionVariant::Return(exp) => {
            return Err(ConstEvalError::CannotBeEvaluatedToConst {
                span: exp.span.clone(),
            });
        }
        ty::TyExpressionVariant::MatchExp { desugared, .. } => {
            const_eval_typed_expr(lookup, known_consts, desugared, allow_configurables)?
        }
        ty::TyExpressionVariant::IntrinsicFunction(kind) => {
            const_eval_intrinsic(lookup, known_consts, kind, allow_configurables)?
        }
        ty::TyExpressionVariant::IfExp {
            condition,
            then,
            r#else,
        } => {
            match const_eval_typed_expr(lookup, known_consts, condition, allow_configurables)? {
                Some(Constant {
                    value: ConstantValue::Bool(cond),
                    ..
                }) => {
                    if cond {
                        const_eval_typed_expr(lookup, known_consts, then, allow_configurables)?
                    } else if let Some(r#else) = r#else {
                        const_eval_typed_expr(lookup, known_consts, r#else, allow_configurables)?
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
                    });
                }
            }
        }
        ty::TyExpressionVariant::CodeBlock(codeblock) => {
            const_eval_codeblock(lookup, known_consts, codeblock, allow_configurables)?
        }
        ty::TyExpressionVariant::ArrayIndex { prefix, index } => {
            let prefix = const_eval_typed_expr(lookup, known_consts, prefix, allow_configurables)?;
            let index = const_eval_typed_expr(lookup, known_consts, index, allow_configurables)?;
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
                        return Err(ConstEvalError::CompileError);
                    }
                }
                _ => {
                    return Err(ConstEvalError::CannotBeEvaluatedToConst {
                        span: expr.span.clone(),
                    });
                }
            }
        }
        ty::TyExpressionVariant::Ref(_) | ty::TyExpressionVariant::Deref(_) => {
            return Err(ConstEvalError::CompileError);
        }
        ty::TyExpressionVariant::EnumTag { exp } => {
            let value = const_eval_typed_expr(lookup, known_consts, exp, allow_configurables)?
                .map(|x| x.value);
            if let Some(ConstantValue::Struct(fields)) = value {
                Some(fields[0].clone())
            } else {
                return Err(ConstEvalError::CompileError);
            }
        }
        ty::TyExpressionVariant::UnsafeDowncast { exp, .. } => {
            let value = const_eval_typed_expr(lookup, known_consts, exp, allow_configurables)?
                .map(|x| x.value);
            if let Some(ConstantValue::Struct(fields)) = value {
                Some(fields[1].clone())
            } else {
                return Err(ConstEvalError::CompileError);
            }
        }
        ty::TyExpressionVariant::WhileLoop {
            condition, body, ..
        } => {
            // Arbitrary limit of iterations to avoid infinite loops like
            // while true {}
            let mut limit = 1_000_000;

            while limit >= 0 {
                limit -= 1;

                let condition =
                    const_eval_typed_expr(lookup, known_consts, condition, allow_configurables)?;
                match condition.map(|x| x.value) {
                    Some(ConstantValue::Bool(true)) => {
                        // Break and continue are not implemented, so there is need for flow control here
                        let _ =
                            const_eval_codeblock(lookup, known_consts, body, allow_configurables)?;
                    }
                    _ => break,
                }
            }

            None
        }
        ty::TyExpressionVariant::Reassignment(r) => {
            let rhs =
                const_eval_typed_expr(lookup, known_consts, &r.rhs, allow_configurables)?.unwrap();
            match &r.lhs {
                ty::TyReassignmentTarget::ElementAccess {
                    base_name, indices, ..
                } => {
                    if !indices.is_empty() {
                        return Err(ConstEvalError::CannotBeEvaluatedToConst {
                            span: expr.span.clone(),
                        });
                    }
                    if let Some(lhs) = known_consts.get_mut(base_name) {
                        *lhs = rhs;
                        return Ok(None);
                    } else {
                        return Err(ConstEvalError::CannotBeEvaluatedToConst {
                            span: expr.span.clone(),
                        });
                    }
                }
                ty::TyReassignmentTarget::Deref(_) => {
                    return Err(ConstEvalError::CannotBeEvaluatedToConst {
                        span: expr.span.clone(),
                    });
                }
            }
        }
        ty::TyExpressionVariant::FunctionParameter
        | ty::TyExpressionVariant::AsmExpression { .. }
        | ty::TyExpressionVariant::LazyOperator { .. }
        | ty::TyExpressionVariant::AbiCast { .. }
        | ty::TyExpressionVariant::StorageAccess(_)
        | ty::TyExpressionVariant::AbiName(_)
        | ty::TyExpressionVariant::Break
        | ty::TyExpressionVariant::Continue
        | ty::TyExpressionVariant::ForLoop { .. } => {
            return Err(ConstEvalError::CannotBeEvaluatedToConst {
                span: expr.span.clone(),
            });
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
    allow_configurables: bool,
) -> Result<Option<Constant>, ConstEvalError> {
    // the current result
    let mut result: Result<Option<Constant>, ConstEvalError> = Ok(None);
    // keep track of new bindings for this codeblock
    let mut bindings: Vec<_> = vec![];

    for ast_node in &codeblock.contents {
        result = match &ast_node.content {
            ty::TyAstNodeContent::Declaration(decl @ ty::TyDecl::VariableDecl(var_decl)) => {
                if let Ok(Some(rhs)) =
                    const_eval_typed_expr(lookup, known_consts, &var_decl.body, allow_configurables)
                {
                    known_consts.push(var_decl.name.clone(), rhs);
                    bindings.push(var_decl.name.clone());
                    Ok(None)
                } else {
                    Err(ConstEvalError::CannotBeEvaluatedToConst {
                        span: decl.span(lookup.engines).clone(),
                    })
                }
            }
            ty::TyAstNodeContent::Declaration(ty::TyDecl::ConstantDecl(const_decl)) => {
                let ty_const_decl = lookup.engines.de().get_constant(&const_decl.decl_id);
                if let Some(constant) = ty_const_decl
                    .value
                    .clone()
                    .and_then(|expr| {
                        const_eval_typed_expr(lookup, known_consts, &expr, allow_configurables).ok()
                    })
                    .flatten()
                {
                    known_consts.push(ty_const_decl.name().clone(), constant);
                    bindings.push(ty_const_decl.name().clone());
                    Ok(None)
                } else {
                    Err(ConstEvalError::CannotBeEvaluatedToConst {
                        span: ty_const_decl.span.clone(),
                    })
                }
            }
            ty::TyAstNodeContent::Declaration(_) => Ok(None),
            ty::TyAstNodeContent::Expression(e) => match e.expression {
                ty::TyExpressionVariant::ImplicitReturn(_) => {
                    if let Ok(Some(constant)) =
                        const_eval_typed_expr(lookup, known_consts, e, allow_configurables)
                    {
                        Ok(Some(constant))
                    } else {
                        Err(ConstEvalError::CannotBeEvaluatedToConst {
                            span: e.span.clone(),
                        })
                    }
                }
                _ => {
                    if const_eval_typed_expr(lookup, known_consts, e, allow_configurables).is_err()
                    {
                        Err(ConstEvalError::CannotBeEvaluatedToConst {
                            span: e.span.clone(),
                        })
                    } else {
                        Ok(None)
                    }
                }
            },
            ty::TyAstNodeContent::SideEffect(_) => Err(ConstEvalError::CannotBeEvaluatedToConst {
                span: ast_node.span.clone(),
            }),
            ty::TyAstNodeContent::Error(_, _) => {
                unreachable!("error node found when generating IR");
            }
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

fn as_encode_buffer(buffer: &Constant) -> Option<(&Vec<u8>, u64)> {
    match &buffer.value {
        ConstantValue::Struct(fields) => {
            let slice = match &fields[0].value {
                ConstantValue::RawUntypedSlice(bytes) => bytes,
                _ => return None,
            };
            let len = match fields[1].value {
                ConstantValue::Uint(v) => v,
                _ => return None,
            };
            Some((slice, len))
        }
        _ => None,
    }
}

fn to_encode_buffer(lookup: &mut LookupEnv, bytes: Vec<u8>, len: u64) -> Constant {
    Constant {
        ty: Type::new_struct(
            lookup.context,
            vec![
                Type::get_slice(lookup.context),
                Type::get_uint64(lookup.context),
            ],
        ),
        value: ConstantValue::Struct(vec![
            Constant {
                ty: Type::get_slice(lookup.context),
                value: ConstantValue::RawUntypedSlice(bytes),
            },
            Constant {
                ty: Type::get_uint64(lookup.context),
                value: ConstantValue::Uint(len),
            },
        ]),
    }
}

fn const_eval_intrinsic(
    lookup: &mut LookupEnv,
    known_consts: &mut MappedStack<Ident, Constant>,
    intrinsic: &TyIntrinsicFunctionKind,
    allow_configurables: bool,
) -> Result<Option<Constant>, ConstEvalError> {
    let mut args = vec![];
    for arg in intrinsic.arguments.iter() {
        if let Ok(Some(constant)) =
            const_eval_typed_expr(lookup, known_consts, arg, allow_configurables)
        {
            args.push(constant);
        } else {
            return Err(ConstEvalError::CannotBeEvaluatedToConst {
                span: arg.span.clone(),
            });
        }
    }

    assert!(args.len() == intrinsic.arguments.len());

    match intrinsic.kind {
        Intrinsic::Add | Intrinsic::Sub | Intrinsic::Mul | Intrinsic::Div | Intrinsic::Mod => {
            let ty = args[0].ty;
            assert!(args.len() == 2 && ty.eq(lookup.context, &args[1].ty));

            use ConstantValue::*;
            match (&args[0].value, &args[1].value) {
                (Uint(arg1), Uint(ref arg2)) => {
                    // All arithmetic is done as if it were u64
                    let result = match intrinsic.kind {
                        Intrinsic::Add => arg1.checked_add(*arg2),
                        Intrinsic::Sub => arg1.checked_sub(*arg2),
                        Intrinsic::Mul => arg1.checked_mul(*arg2),
                        Intrinsic::Div => arg1.checked_div(*arg2),
                        Intrinsic::Mod => arg1.checked_rem(*arg2),
                        _ => unreachable!(),
                    };

                    match result {
                        Some(result) => Ok(Some(Constant {
                            ty,
                            value: ConstantValue::Uint(result),
                        })),
                        None => Err(ConstEvalError::CannotBeEvaluatedToConst {
                            span: intrinsic.span.clone(),
                        }),
                    }
                }
                (U256(arg1), U256(arg2)) => {
                    let result = match intrinsic.kind {
                        Intrinsic::Add => arg1.checked_add(arg2),
                        Intrinsic::Sub => arg1.checked_sub(arg2),
                        Intrinsic::Mul => arg1.checked_mul(arg2),
                        Intrinsic::Div => arg1.checked_div(arg2),
                        Intrinsic::Mod => Some(arg1.rem(arg2)),
                        _ => unreachable!(),
                    };

                    match result {
                        Some(result) => Ok(Some(Constant {
                            ty,
                            value: ConstantValue::U256(result),
                        })),
                        None => Err(ConstEvalError::CannotBeEvaluatedToConst {
                            span: intrinsic.span.clone(),
                        }),
                    }
                }
                _ => {
                    panic!("Type checker allowed incorrect args to binary op");
                }
            }
        }
        Intrinsic::And | Intrinsic::Or | Intrinsic::Xor => {
            let ty = args[0].ty;
            assert!(args.len() == 2 && ty.eq(lookup.context, &args[1].ty));

            use ConstantValue::*;
            match (&args[0].value, &args[1].value) {
                (Uint(arg1), Uint(ref arg2)) => {
                    // All arithmetic is done as if it were u64
                    let result = match intrinsic.kind {
                        Intrinsic::And => Some(arg1.bitand(arg2)),
                        Intrinsic::Or => Some(arg1.bitor(*arg2)),
                        Intrinsic::Xor => Some(arg1.bitxor(*arg2)),
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
                (U256(arg1), U256(arg2)) => {
                    let result = match intrinsic.kind {
                        Intrinsic::And => Some(arg1.bitand(arg2)),
                        Intrinsic::Or => Some(arg1.bitor(arg2)),
                        Intrinsic::Xor => Some(arg1.bitxor(arg2)),
                        _ => unreachable!(),
                    };

                    match result {
                        Some(sum) => Ok(Some(Constant {
                            ty,
                            value: ConstantValue::U256(sum),
                        })),
                        None => Err(ConstEvalError::CannotBeEvaluatedToConst {
                            span: intrinsic.span.clone(),
                        }),
                    }
                }
                (B256(arg1), B256(arg2)) => {
                    let result = match intrinsic.kind {
                        Intrinsic::And => Some(arg1.bitand(arg2)),
                        Intrinsic::Or => Some(arg1.bitor(arg2)),
                        Intrinsic::Xor => Some(arg1.bitxor(arg2)),
                        _ => unreachable!(),
                    };

                    match result {
                        Some(result) => Ok(Some(Constant {
                            ty,
                            value: ConstantValue::B256(result),
                        })),
                        None => Err(ConstEvalError::CannotBeEvaluatedToConst {
                            span: intrinsic.span.clone(),
                        }),
                    }
                }
                _ => {
                    panic!("Type checker allowed incorrect args to binary op");
                }
            }
        }
        Intrinsic::Lsh | Intrinsic::Rsh => {
            assert!(args.len() == 2);
            assert!(args[0].ty.is_uint(lookup.context) || args[0].ty.is_b256(lookup.context));
            assert!(args[1].ty.is_uint64(lookup.context));

            let ty = args[0].ty;

            use ConstantValue::*;
            match (&args[0].value, &args[1].value) {
                (Uint(arg1), Uint(ref arg2)) => {
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
                (U256(arg1), Uint(ref arg2)) => {
                    let result = match intrinsic.kind {
                        Intrinsic::Lsh => arg1.checked_shl(arg2),
                        Intrinsic::Rsh => Some(arg1.shr(arg2)),
                        _ => unreachable!(),
                    };

                    match result {
                        Some(value) => Ok(Some(Constant {
                            ty,
                            value: ConstantValue::U256(value),
                        })),
                        None => Err(ConstEvalError::CannotBeEvaluatedToConst {
                            span: intrinsic.span.clone(),
                        }),
                    }
                }
                (B256(arg1), Uint(ref arg2)) => {
                    let result = match intrinsic.kind {
                        Intrinsic::Lsh => arg1.checked_shl(arg2),
                        Intrinsic::Rsh => Some(arg1.shr(arg2)),
                        _ => unreachable!(),
                    };

                    match result {
                        Some(result) => Ok(Some(Constant {
                            ty,
                            value: ConstantValue::B256(result),
                        })),
                        None => Err(ConstEvalError::CannotBeEvaluatedToConst {
                            span: intrinsic.span.clone(),
                        }),
                    }
                }
                _ => {
                    panic!("Type checker allowed incorrect args to binary op");
                }
            }
        }
        Intrinsic::SizeOfType => {
            let targ = &intrinsic.type_arguments[0];
            let ir_type = convert_resolved_typeid(
                lookup.engines.te(),
                lookup.engines.de(),
                lookup.context,
                &targ.type_id,
                &targ.span,
            )
            .map_err(|_| ConstEvalError::CompileError)?;
            Ok(Some(Constant {
                ty: Type::get_uint64(lookup.context),
                value: ConstantValue::Uint(ir_type.size(lookup.context).in_bytes()),
            }))
        }
        Intrinsic::SizeOfVal => {
            let val = &intrinsic.arguments[0];
            let type_id = val.return_type;
            let ir_type = convert_resolved_typeid(
                lookup.engines.te(),
                lookup.engines.de(),
                lookup.context,
                &type_id,
                &val.span,
            )
            .map_err(|_| ConstEvalError::CompileError)?;
            Ok(Some(Constant {
                ty: Type::get_uint64(lookup.context),
                value: ConstantValue::Uint(ir_type.size(lookup.context).in_bytes()),
            }))
        }
        Intrinsic::SizeOfStr => {
            let targ = &intrinsic.type_arguments[0];
            let ir_type = convert_resolved_typeid(
                lookup.engines.te(),
                lookup.engines.de(),
                lookup.context,
                &targ.type_id,
                &targ.span,
            )
            .map_err(|_| ConstEvalError::CompileError)?;
            Ok(Some(Constant {
                ty: Type::get_uint64(lookup.context),
                value: ConstantValue::Uint(
                    ir_type.get_string_len(lookup.context).unwrap_or_default(),
                ),
            }))
        }
        Intrinsic::AssertIsStrArray => {
            let targ = &intrinsic.type_arguments[0];
            let ir_type = convert_resolved_typeid(
                lookup.engines.te(),
                lookup.engines.de(),
                lookup.context,
                &targ.type_id,
                &targ.span,
            )
            .map_err(|_| ConstEvalError::CompileError)?;
            match ir_type.get_content(lookup.context) {
                TypeContent::StringSlice | TypeContent::StringArray(_) => Ok(Some(Constant {
                    ty: Type::get_unit(lookup.context),
                    value: ConstantValue::Unit,
                })),
                _ => Err(ConstEvalError::CompileError),
            }
        }
        Intrinsic::ToStrArray => {
            assert!(args.len() == 1);
            match &args[0].value {
                ConstantValue::String(s) => {
                    Ok(Some(Constant::new_string(lookup.context, s.to_vec())))
                }
                _ => {
                    unreachable!("Type checker allowed non string value for ToStrArray")
                }
            }
        }
        Intrinsic::Eq => {
            assert!(args.len() == 2);
            Ok(Some(Constant {
                ty: Type::get_bool(lookup.context),
                value: ConstantValue::Bool(args[0].eq(lookup.context, &args[1])),
            }))
        }
        Intrinsic::Gt => match (&args[0].value, &args[1].value) {
            (ConstantValue::Uint(val1), ConstantValue::Uint(val2)) => Ok(Some(Constant {
                ty: Type::get_bool(lookup.context),
                value: ConstantValue::Bool(val1 > val2),
            })),
            (ConstantValue::U256(val1), ConstantValue::U256(val2)) => Ok(Some(Constant {
                ty: Type::get_bool(lookup.context),
                value: ConstantValue::Bool(val1 > val2),
            })),
            _ => {
                unreachable!("Type checker allowed non integer value for GreaterThan")
            }
        },
        Intrinsic::Lt => match (&args[0].value, &args[1].value) {
            (ConstantValue::Uint(val1), ConstantValue::Uint(val2)) => Ok(Some(Constant {
                ty: Type::get_bool(lookup.context),
                value: ConstantValue::Bool(val1 < val2),
            })),
            (ConstantValue::U256(val1), ConstantValue::U256(val2)) => Ok(Some(Constant {
                ty: Type::get_bool(lookup.context),
                value: ConstantValue::Bool(val1 < val2),
            })),
            _ => {
                unreachable!("Type checker allowed non integer value for LessThan")
            }
        },
        Intrinsic::AddrOf
        | Intrinsic::PtrAdd
        | Intrinsic::PtrSub
        | Intrinsic::IsReferenceType
        | Intrinsic::IsStrArray
        | Intrinsic::Gtf
        | Intrinsic::StateClear
        | Intrinsic::StateLoadWord
        | Intrinsic::StateStoreWord
        | Intrinsic::StateLoadQuad
        | Intrinsic::StateStoreQuad
        | Intrinsic::Log
        | Intrinsic::Revert
        | Intrinsic::JmpMem
        | Intrinsic::Smo => Err(ConstEvalError::CannotBeEvaluatedToConst {
            span: intrinsic.span.clone(),
        }),
        Intrinsic::Not => {
            // `not` works only with uint/u256/b256 at the moment
            // `bool` ops::Not implementation uses `__eq`.

            assert!(args.len() == 1);
            assert!(args[0].ty.is_uint(lookup.context) || args[0].ty.is_b256(lookup.context));

            let Some(arg) = args.into_iter().next() else {
                unreachable!("Unexpected 'not' without any arguments");
            };

            match arg.value {
                ConstantValue::Uint(n) => {
                    let n = match arg.ty.get_uint_width(lookup.context) {
                        Some(8) => !(n as u8) as u64,
                        Some(16) => !(n as u16) as u64,
                        Some(32) => !(n as u32) as u64,
                        Some(64) => !n,
                        _ => unreachable!("Invalid unsigned integer width"),
                    };
                    Ok(Some(Constant {
                        ty: arg.ty,
                        value: ConstantValue::Uint(n),
                    }))
                }
                ConstantValue::U256(n) => Ok(Some(Constant {
                    ty: arg.ty,
                    value: ConstantValue::U256(n.not()),
                })),
                ConstantValue::B256(v) => Ok(Some(Constant {
                    ty: arg.ty,
                    value: ConstantValue::B256(v.not()),
                })),
                _ => {
                    unreachable!("Type checker allowed non integer value for Not");
                }
            }
        }
        Intrinsic::ContractCall | Intrinsic::ContractRet => {
            Err(ConstEvalError::CannotBeEvaluatedToConst {
                span: intrinsic.span.clone(),
            })
        }
        Intrinsic::EncodeBufferEmpty => Ok(Some(to_encode_buffer(lookup, vec![], 0))),
        Intrinsic::EncodeBufferAppend => {
            assert!(args.len() == 2);

            let (slice, mut len) = as_encode_buffer(&args[0]).unwrap();
            let mut bytes = slice.clone();

            use ConstantValue::*;
            match &args[1].value {
                Bool(v) => {
                    bytes.extend(if *v { [1] } else { [0] });
                    len += 1;
                    Ok(Some(to_encode_buffer(lookup, bytes, len)))
                }
                Uint(v) => {
                    match &*lookup.engines.te().get(intrinsic.arguments[1].return_type) {
                        TypeInfo::UnsignedInteger(IntegerBits::Eight) => {
                            bytes.extend((*v as u8).to_be_bytes());
                            len += 1;
                        }
                        TypeInfo::UnsignedInteger(IntegerBits::Sixteen) => {
                            bytes.extend((*v as u16).to_be_bytes());
                            len += 2;
                        }
                        TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo) => {
                            bytes.extend((*v as u32).to_be_bytes());
                            len += 4;
                        }
                        TypeInfo::UnsignedInteger(IntegerBits::SixtyFour) => {
                            bytes.extend(v.to_be_bytes());
                            len += 8;
                        }
                        _ => {
                            return Err(ConstEvalError::CannotBeEvaluatedToConst {
                                span: intrinsic.span.clone(),
                            });
                        }
                    };
                    Ok(Some(to_encode_buffer(lookup, bytes, len)))
                }
                U256(v) => {
                    bytes.extend(v.to_be_bytes());
                    len += 32;
                    Ok(Some(to_encode_buffer(lookup, bytes, len)))
                }
                B256(v) => {
                    bytes.extend(v.to_be_bytes());
                    len += 32;
                    Ok(Some(to_encode_buffer(lookup, bytes, len)))
                }
                String(v) => {
                    if let TypeInfo::StringSlice =
                        &*lookup.engines.te().get(intrinsic.arguments[1].return_type)
                    {
                        let l = v.len() as u64;
                        bytes.extend(l.to_be_bytes());
                        len += 8;
                    }

                    bytes.extend(v);
                    len += v.len() as u64;

                    Ok(Some(to_encode_buffer(lookup, bytes, len)))
                }
                _ => Err(ConstEvalError::CannotBeEvaluatedToConst {
                    span: intrinsic.span.clone(),
                }),
            }
        }
        Intrinsic::EncodeBufferAsRawSlice => {
            assert!(args.len() == 1);

            let (slice, len) = as_encode_buffer(&args[0]).unwrap();
            let bytes = slice.clone();

            Ok(Some(Constant {
                ty: Type::get_slice(lookup.context),
                value: ConstantValue::RawUntypedSlice(bytes[0..(len as usize)].to_vec()),
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
        let mut context = Context::new(
            engines.se(),
            sway_ir::ExperimentalFlags {
                new_encoding: false,
            },
        );
        let mut md_mgr = MetadataManager::default();
        let core_lib = namespace::Root::from(namespace::Module {
            name: Some(sway_types::Ident::new_no_span(
                "assert_is_constant_test".to_string(),
            )),
            ..Default::default()
        });

        let r = crate::compile_to_ast(
            &handler,
            &engines,
            std::sync::Arc::from(format!("library; {prefix} fn f() -> u64 {{ {expr}; 0 }}")),
            core_lib,
            None,
            "test",
            None,
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
                ty::TyDecl::FunctionDecl(x) => {
                    if engines.de().get_function(&x.decl_id).name.as_str() == "f" {
                        Some(x)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .expect("An function named `f` was not found.");

        let f = engines.de().get_function(&f.decl_id);
        let expr_under_test = f.body.contents.first().unwrap();

        let expr_under_test = match &expr_under_test.content {
            ty::TyAstNodeContent::Expression(expr_under_test) => expr_under_test.clone(),
            ty::TyAstNodeContent::Declaration(crate::language::ty::TyDecl::ConstantDecl(decl)) => {
                let decl = engines.de().get_constant(&decl.decl_id);
                decl.value.clone().unwrap()
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
            false,
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
        assert_is_constant(
            true,
            "struct Person { age: u64 }",
            "Person { age: { let mut x = 0; x = 1; 1 } }",
        );
        assert_is_constant(true, "", "if true { 1 } else { 0 }");
        assert_is_constant(true, "", "(0,1).0");
        assert_is_constant(true, "", "[0,1][0]");

        // u256
        assert_is_constant(
            true,
            "",
            "0x0000000000000000000000000000000000000000000000000000000000000001u256",
        );
        assert_is_constant(
            true,
            "",
            "__add(0x0000000000000000000000000000000000000000000000000000000000000001u256, 0x0000000000000000000000000000000000000000000000000000000000000001u256)",
        );
        assert_is_constant(
            true,
            "",
            "__eq(0x0000000000000000000000000000000000000000000000000000000000000001u256, 0x0000000000000000000000000000000000000000000000000000000000000001u256)",
        );
        assert_is_constant(
            true,
            "",
            "__gt(0x0000000000000000000000000000000000000000000000000000000000000001u256, 0x0000000000000000000000000000000000000000000000000000000000000001u256)",
        );
        assert_is_constant(
            true,
            "",
            "__lt(0x0000000000000000000000000000000000000000000000000000000000000001u256, 0x0000000000000000000000000000000000000000000000000000000000000001u256)",
        );
        assert_is_constant(
            true,
            "",
            "__lsh(0x0000000000000000000000000000000000000000000000000000000000000001u256, 2)",
        );
        assert_is_constant(
            true,
            "",
            "__not(0x0000000000000000000000000000000000000000000000000000000000000001u256)",
        );

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
