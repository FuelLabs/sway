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
use sway_types::{ident::Ident, span::Spanned};
use sway_utils::mapped_stack::MappedStack;

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

            let function_decl = lookup.engines.de().get_function(fn_ref);
            let res = const_eval_codeblock(lookup, known_consts, &function_decl.body)?;
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
                }
            }
            if field_vals.len() < fields.len() {
                // We couldn't evaluate all fields to a constant.
                return Ok(None);
            }
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
                }
            }
            if element_vals.len() < contents.len() || element_typs.is_empty() {
                // We couldn't evaluate all fields to a constant or cannot determine element type.
                return Ok(None);
            }
            let elem_type_info = lookup.engines.te().get(*elem_type);
            if !element_typs.iter().all(|tid| {
                lookup
                    .engines
                    .te()
                    .get(*tid)
                    .eq(&elem_type_info, lookup.engines)
            }) {
                // This shouldn't happen if the type checker did its job.
                return Ok(None);
            }
            create_array_aggregate(
                lookup.engines.te(),
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
        // we could allow non-local control flow in pure functions, but it would
        // require some more work and at this point it's not clear if it is too useful
        // for constant initializers -- the user can always refactor their pure functions
        // to not use the return statement
        ty::TyExpressionVariant::Return(_exp) => None,
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
                _ => None,
            }
        }
        ty::TyExpressionVariant::CodeBlock(codeblock) => {
            const_eval_codeblock(lookup, known_consts, codeblock)?
        }
        ty::TyExpressionVariant::ArrayIndex { .. }
        | ty::TyExpressionVariant::Reassignment(_)
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
        | ty::TyExpressionVariant::WhileLoop { .. } => None,
    })
}

// the (constant) value of a codeblock is essentially it's last expression if there is one
// or if it makes sense as the last expression, e.g. a dangling let-expression in a codeblock
// would be an evaluation error
fn const_eval_codeblock(
    lookup: &mut LookupEnv,
    known_consts: &mut MappedStack<Ident, Constant>,
    codeblock: &ty::TyCodeBlock,
) -> Result<Option<Constant>, CompileError> {
    // the current result
    let mut res_const = None;
    // keep track of new bindings for this codeblock
    let mut bindings: Vec<_> = vec![];

    for ast_node in &codeblock.contents {
        match &ast_node.content {
            ty::TyAstNodeContent::Declaration(ty::TyDecl::VariableDecl(var_decl)) => {
                let rhs_opt = const_eval_typed_expr(lookup, known_consts, &var_decl.body)?;
                if let Some(rhs) = rhs_opt {
                    known_consts.push(var_decl.name.clone(), rhs);
                    bindings.push(var_decl.name.clone());
                }
                res_const = None
            }
            ty::TyAstNodeContent::Declaration(ty::TyDecl::ConstantDecl(const_decl)) => {
                let ty_const_decl = lookup.engines.de().get_constant(&const_decl.decl_id);
                if let Some(const_expr) = ty_const_decl.value {
                    if let Some(constant) =
                        const_eval_typed_expr(lookup, known_consts, &const_expr)?
                    {
                        known_consts.push(const_decl.name.clone(), constant);
                        bindings.push(const_decl.name.clone());
                    }
                }
                res_const = None
            }
            ty::TyAstNodeContent::Declaration(_) => res_const = None,
            ty::TyAstNodeContent::Expression(e)
            | ty::TyAstNodeContent::ImplicitReturnExpression(e) => {
                res_const = const_eval_typed_expr(lookup, known_consts, e)?
            }
            ty::TyAstNodeContent::SideEffect(_) => res_const = None,
        }
    }
    // remove introduced vars/consts from scope at the end of the codeblock
    for name in bindings {
        known_consts.pop(&name)
    }
    Ok(res_const)
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
        | sway_ast::Intrinsic::Xor
        | sway_ast::Intrinsic::Mod => {
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
                None => Ok(None),
            }
        }
        sway_ast::Intrinsic::Lsh | sway_ast::Intrinsic::Rsh => {
            let ty = args[0].ty;
            assert!(
                args.len() == 2
                    && ty.is_uint(lookup.context)
                    && args[1].ty.is_uint64(lookup.context)
            );
            let (ConstantValue::Uint(arg1), ConstantValue::Uint(ref arg2)) = (&args[0].value, &args[1].value)
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
                None => Ok(None),
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
                lookup.engines.te(),
                lookup.engines.de(),
                lookup.context,
                &type_id,
                &val.span,
            )?;
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
            )?;
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
        sway_ast::Intrinsic::IsReferenceType
        | sway_ast::Intrinsic::IsStrType
        | sway_ast::Intrinsic::Gtf
        | sway_ast::Intrinsic::StateClear
        | sway_ast::Intrinsic::StateLoadWord
        | sway_ast::Intrinsic::StateStoreWord
        | sway_ast::Intrinsic::StateLoadQuad
        | sway_ast::Intrinsic::StateStoreQuad
        | sway_ast::Intrinsic::Log
        | sway_ast::Intrinsic::Revert
        | sway_ast::Intrinsic::Smo => Ok(None),
        sway_ast::Intrinsic::Not => {
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
