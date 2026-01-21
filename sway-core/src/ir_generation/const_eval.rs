use std::{
    hash::{DefaultHasher, Hash},
    io::Read,
    ops::{BitAnd, BitOr, BitXor, Not, Rem},
};

use crate::{
    TypeInfo, UnifyCheck, engine_threading::*, ir_generation::function::{get_encoding_id, get_encoding_representation_by_id, get_memory_id}, language::{
        CallPath, LazyOp, Literal, ty::{self, ProjectionKind, TyConstantDecl, TyIntrinsicFunctionKind}
    }, metadata::MetadataManager, semantic_analysis::*
};

use super::{
    convert::{convert_literal_to_constant, convert_resolved_type_id},
    function::FnCompiler,
    types::*,
};

use hashbrown::HashSet;
use sway_ast::Intrinsic;
use sway_error::error::CompileError;
use sway_ir::{
    constant::{ConstantContent, ConstantValue},
    context::Context,
    module::Module,
    value::Value,
    Constant, GlobalVar, InstOp, Instruction, Type, TypeContent,
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
                return Ok(Some(Value::new_constant(env.context, *constant)));
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
                return Ok(Some(Value::new_constant(env.context, *constant)));
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
            .get_global_variable(env.context, &call_path.as_vec_string()),
        env.module_ns,
    ) {
        (Some(global_var), _) => {
            let constant = global_var
                .get_initializer(env.context)
                .expect("const decl without initializer, should've been detected way early");
            Ok(Some(Value::new_constant(env.context, *constant)))
        }
        (None, Some(module_ns)) => {
            // See if we it's a global const and whether we can compile it *now*.
            let decl = module_ns.root_items().check_symbol(&call_path.suffix);
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
                        call_path, value, ..
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
                        &value,
                    )?;

                    let const_val_c = *const_val
                        .get_constant(env.context)
                        .expect("Must have been compiled to a constant");

                    let c_ty = const_val_c.get_content(env.context).ty;
                    let const_global = GlobalVar::new(env.context, c_ty, Some(const_val_c), false);

                    env.module.add_global_variable(
                        env.context,
                        call_path.as_vec_string().to_vec(),
                        const_global,
                    );

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
    const_expr: &ty::TyExpression,
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

    Ok(Value::new_constant(context, constant_evaluated).add_metadatum(context, span_id_idx))
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
            let span = call_path.span();
            let span = if span.is_dummy() {
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

    if contains_outer_vars(const_expr, &HashSet::new()) {
        err
    } else {
        let mut known_consts = MappedStack::<Ident, Constant>::new();
        match const_eval_typed_expr(lookup, &mut known_consts, const_expr) {
            Ok(Some(constant)) => Ok(constant),
            Ok(None) => err,
            Err(_) => err,
        }
    }
}

fn create_array_from_vec(
    lookup: &mut LookupEnv,
    elem_type: crate::TypeId,
    element_types: Vec<crate::TypeId>,
    element_vals: Vec<Constant>,
) -> Option<Constant> {
    assert!({
        let unify_check = UnifyCheck::coercion(lookup.engines);
        element_types
            .iter()
            .all(|tid| unify_check.check(*tid, elem_type))
    });

    let arr = create_array_aggregate(
        lookup.engines,
        lookup.context,
        lookup.md_mgr,
        lookup.module,
        elem_type,
        element_types.len().try_into().unwrap(),
    )
    .map_or(None, |array_ty| {
        Some(ConstantContent::new_array(
            lookup.context,
            array_ty.get_array_elem_type(lookup.context).unwrap(),
            element_vals
                .iter()
                .map(|f| f.get_content(lookup.context).clone())
                .collect(),
        ))
    });

    arr.map(|c| Constant::unique(lookup.context, c))
}

/// Returns true if the `expr` contains any variables declared outside of itself.
///
/// `in_expr_vars` are variables defined within the `expr` in a scope being examined.
fn contains_outer_vars(expr: &ty::TyExpression, in_expr_vars: &HashSet<&Ident>) -> bool {
    match &expr.expression {
        ty::TyExpressionVariant::ConstGenericExpression {
            decl,
            span: _,
            call_path: _,
        } => decl
            .value
            .as_ref()
            .is_some_and(|value| contains_outer_vars(value, in_expr_vars)),
        ty::TyExpressionVariant::Literal(_) => false,
        ty::TyExpressionVariant::FunctionApplication {
            arguments,
            fn_ref: _,
            call_path: _,
            selector,
            type_binding: _,
            method_target: _,
            contract_call_params,
            contract_caller,
        } => {
            arguments
                .iter()
                .any(|(_ident, arg)| contains_outer_vars(arg, in_expr_vars))
                || selector.as_ref().is_some_and(|selector| {
                    contains_outer_vars(&selector.contract_address, in_expr_vars)
                        || contains_outer_vars(&selector.contract_caller, in_expr_vars)
                })
                || contract_call_params
                    .values()
                    .any(|param| contains_outer_vars(param, in_expr_vars))
                || contract_caller
                    .as_ref()
                    .is_some_and(|caller| contains_outer_vars(caller, in_expr_vars))
        }
        ty::TyExpressionVariant::ConstantExpression {
            decl: _,
            span: _,
            call_path: _,
        } => false,
        ty::TyExpressionVariant::ConfigurableExpression { .. } => false,
        ty::TyExpressionVariant::VariableExpression { name, .. } => !in_expr_vars.contains(name),
        ty::TyExpressionVariant::StructExpression {
            fields,
            instantiation_span: _,
            struct_id: _,
            call_path_binding: _,
        } => fields
            .iter()
            .any(|field| contains_outer_vars(&field.value, in_expr_vars)),
        ty::TyExpressionVariant::Tuple { fields } => fields
            .iter()
            .any(|field| contains_outer_vars(field, in_expr_vars)),
        ty::TyExpressionVariant::ArrayExplicit {
            elem_type: _,
            contents,
        } => contents
            .iter()
            .any(|elem| contains_outer_vars(elem, in_expr_vars)),
        ty::TyExpressionVariant::ArrayRepeat {
            elem_type: _,
            value,
            length,
        } => contains_outer_vars(value, in_expr_vars) || contains_outer_vars(length, in_expr_vars),
        ty::TyExpressionVariant::EnumInstantiation {
            enum_ref: _,
            tag: _,
            contents,
            variant_instantiation_span: _,
            variant_name: _,
            call_path_binding: _,
            call_path_decl: _,
        } => contents
            .as_ref()
            .is_some_and(|value| contains_outer_vars(value, in_expr_vars)),
        ty::TyExpressionVariant::StructFieldAccess {
            prefix,
            field_to_access: _,
            resolved_type_of_parent: _,
            field_instantiation_span: _,
        } => contains_outer_vars(prefix, in_expr_vars),
        ty::TyExpressionVariant::TupleElemAccess {
            prefix,
            elem_to_access_num: _,
            resolved_type_of_parent: _,
            elem_to_access_span: _,
        } => contains_outer_vars(prefix, in_expr_vars),
        ty::TyExpressionVariant::ImplicitReturn(exp) => contains_outer_vars(exp, in_expr_vars),
        ty::TyExpressionVariant::Return(exp) => contains_outer_vars(exp, in_expr_vars),
        ty::TyExpressionVariant::Panic(exp) => contains_outer_vars(exp, in_expr_vars),
        ty::TyExpressionVariant::MatchExp {
            desugared,
            scrutinees: _,
        } => contains_outer_vars(desugared, in_expr_vars),
        ty::TyExpressionVariant::IntrinsicFunction(kind) => kind
            .arguments
            .iter()
            .any(|arg| contains_outer_vars(arg, in_expr_vars)),
        ty::TyExpressionVariant::IfExp {
            condition,
            then,
            r#else,
        } => {
            contains_outer_vars(condition, in_expr_vars)
                || contains_outer_vars(then, in_expr_vars)
                || r#else
                    .as_ref()
                    .map_or(false, |e| contains_outer_vars(e, in_expr_vars))
        }
        ty::TyExpressionVariant::CodeBlock(codeblock) => {
            codeblock_contains_outer_vars(codeblock, in_expr_vars.clone())
        }
        ty::TyExpressionVariant::ArrayIndex { prefix, index } => {
            contains_outer_vars(prefix, in_expr_vars) || contains_outer_vars(index, in_expr_vars)
        }
        ty::TyExpressionVariant::Ref(exp) => contains_outer_vars(exp, in_expr_vars),
        ty::TyExpressionVariant::Deref(exp) => contains_outer_vars(exp, in_expr_vars),
        ty::TyExpressionVariant::EnumTag { exp } => contains_outer_vars(exp, in_expr_vars),
        ty::TyExpressionVariant::UnsafeDowncast {
            exp,
            variant: _,
            call_path_decl: _,
        } => contains_outer_vars(exp, in_expr_vars),
        ty::TyExpressionVariant::WhileLoop { condition, body } => {
            contains_outer_vars(condition, in_expr_vars)
                || codeblock_contains_outer_vars(body, in_expr_vars.clone())
        }
        ty::TyExpressionVariant::Reassignment(r) => {
            fn projection_kind_contains_outer_vars<'a>(
                proj_kind: &'a ProjectionKind,
                in_expr_vars: &HashSet<&'a Ident>,
            ) -> bool {
                match proj_kind {
                    ProjectionKind::ArrayIndex {
                        index,
                        index_span: _,
                    } => contains_outer_vars(index, in_expr_vars),
                    ProjectionKind::StructField {
                        name: _,
                        field_to_access: _,
                    } => false,
                    ProjectionKind::TupleField {
                        index: _,
                        index_span: _,
                    } => false,
                }
            }

            contains_outer_vars(&r.rhs, in_expr_vars) || {
                match &r.lhs {
                    ty::TyReassignmentTarget::ElementAccess {
                        base_name,
                        base_type: _,
                        indices,
                    } => {
                        !in_expr_vars.contains(base_name)
                            || indices.iter().any(|proj_kind| {
                                projection_kind_contains_outer_vars(proj_kind, in_expr_vars)
                            })
                    }
                    ty::TyReassignmentTarget::DerefAccess { exp, indices } => {
                        contains_outer_vars(exp, in_expr_vars)
                            || indices.iter().any(|proj_kind| {
                                projection_kind_contains_outer_vars(proj_kind, in_expr_vars)
                            })
                    }
                }
            }
        }
        ty::TyExpressionVariant::AsmExpression {
            registers,
            body: _,
            returns: _,
            whole_block_span: _,
        } => registers.iter().any(|reg| {
            reg.initializer
                .as_ref()
                .is_some_and(|init| contains_outer_vars(init, in_expr_vars))
        }),
        ty::TyExpressionVariant::LazyOperator { op: _, lhs, rhs } => {
            contains_outer_vars(lhs, in_expr_vars) || contains_outer_vars(rhs, in_expr_vars)
        }
        ty::TyExpressionVariant::AbiCast {
            abi_name: _,
            address,
            span: _,
        } => contains_outer_vars(address, in_expr_vars),
        ty::TyExpressionVariant::StorageAccess(storage_access) => storage_access
            .key_expression
            .as_ref()
            .is_some_and(|e| contains_outer_vars(e, in_expr_vars)),
        ty::TyExpressionVariant::AbiName(_) => false,
        ty::TyExpressionVariant::FunctionParameter => false,
        ty::TyExpressionVariant::Break => false,
        ty::TyExpressionVariant::Continue => false,
        ty::TyExpressionVariant::ForLoop { desugared } => {
            contains_outer_vars(desugared, in_expr_vars)
        }
    }
}

fn codeblock_contains_outer_vars<'a>(
    codeblock: &'a ty::TyCodeBlock,
    mut in_expr_vars: HashSet<&'a sway_types::BaseIdent>,
) -> bool {
    for node in codeblock.contents.iter() {
        match &node.content {
            ty::TyAstNodeContent::Declaration(decl) => match decl {
                ty::TyDecl::VariableDecl(var_decl) => {
                    if contains_outer_vars(&var_decl.body, &in_expr_vars) {
                        return true;
                    }
                    in_expr_vars.insert(&var_decl.name);
                }
                // Note that we don't need to check the body of constant declaration.
                // The fact that it has passed the const eval phase already
                // means that it's const-evaluable and, thus, does not contain any
                // outer variables.
                ty::TyDecl::ConstantDecl(_) => {}
                // Other declarations cannot contain outer variables.
                // That's guaranteed by the language semantics.
                _ => {}
            },
            ty::TyAstNodeContent::Expression(e) => {
                if contains_outer_vars(e, &in_expr_vars) {
                    return true;
                }
            }
            ty::TyAstNodeContent::SideEffect(_) => {}
            ty::TyAstNodeContent::Error(_, _) => {}
        }
    }
    false
}

/// Given an environment mapping names to constants,
/// attempt to evaluate a typed expression to a constant.
fn const_eval_typed_expr(
    lookup: &mut LookupEnv,
    known_consts: &mut MappedStack<Ident, Constant>,
    expr: &ty::TyExpression,
) -> Result<Option<Constant>, ConstEvalError> {
    if let TypeInfo::ErrorRecovery(_) = &*lookup.engines.te().get(expr.return_type) {
        return Err(ConstEvalError::CannotBeEvaluatedToConst {
            span: expr.span.clone(),
        });
    }

    Ok(match &expr.expression {
        ty::TyExpressionVariant::ConstGenericExpression { decl, .. } => {
            assert!(decl.value.is_some());
            const_eval_typed_expr(lookup, known_consts, decl.value.as_ref().unwrap())?
        }
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
    
            if function_decl.is_trait_method_dummy {
                return Err(ConstEvalError::CompileError);
            }
    
            let res = const_eval_codeblock(lookup, known_consts, &function_decl.body);
    
            for (name, _) in arguments {
                known_consts.pop(name);
            }
    
            res?
        }
        ty::TyExpressionVariant::ConstantExpression { decl, .. } => {
            let call_path = &decl.call_path;
            let name = &call_path.suffix;
            match known_consts.get(name) {
                Some(constant) => Some(*constant),
                None => (lookup.lookup)(lookup, call_path, &Some(*decl.clone()))
                    .ok()
                    .flatten()
                    .and_then(|v| v.get_constant(lookup.context).cloned()),
            }
        }
        ty::TyExpressionVariant::ConfigurableExpression { span, .. } => {
            return Err(ConstEvalError::CannotBeEvaluatedToConst { span: span.clone() });
        }
        ty::TyExpressionVariant::VariableExpression {
            name, call_path, ..
        } => match known_consts.get(name) {
            // 1. Check if name/call_path is in known_consts.
            Some(cvs) => Some(*cvs),
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
            let (mut field_types, mut field_vals): (Vec<_>, Vec<_>) = (vec![], vec![]);
    
            for field in fields {
                let ty::TyStructExpressionField { name: _, value, .. } = field;
                let eval_expr_opt = const_eval_typed_expr(lookup, known_consts, value)?;
                if let Some(cv) = eval_expr_opt {
                    field_types.push(value.return_type);
                    field_vals.push(cv);
                } else {
                    return Err(ConstEvalError::CannotBeEvaluatedToConst {
                        span: instantiation_span.clone(),
                    });
                }
            }
    
            assert!(field_types.len() == fields.len());
            assert!(field_vals.len() == fields.len());
    
            get_struct_for_types(
                lookup.engines,
                lookup.context,
                lookup.md_mgr,
                lookup.module,
                &field_types,
            )
            .map_or(None, |struct_ty| {
                let c = ConstantContent::new_struct(
                    lookup.context,
                    struct_ty.get_field_types(lookup.context),
                    field_vals
                        .iter()
                        .map(|fv| fv.get_content(lookup.context).clone())
                        .collect(),
                );
                let c = Constant::unique(lookup.context, c);
                Some(c)
            })
        }
        ty::TyExpressionVariant::Tuple { fields } => {
            let (mut field_types, mut field_vals): (Vec<_>, Vec<_>) = (vec![], vec![]);
    
            for value in fields {
                let eval_expr_opt = const_eval_typed_expr(lookup, known_consts, value)?;
                if let Some(cv) = eval_expr_opt {
                    field_types.push(value.return_type);
                    field_vals.push(cv);
                } else {
                    return Err(ConstEvalError::CannotBeEvaluatedToConst {
                        span: expr.span.clone(),
                    });
                }
            }
    
            assert!(field_types.len() == fields.len());
            assert!(field_vals.len() == fields.len());
    
            create_tuple_aggregate(
                lookup.engines,
                lookup.context,
                lookup.md_mgr,
                lookup.module,
                &field_types,
            )
            .map_or(None, |tuple_ty| {
                let c = ConstantContent::new_struct(
                    lookup.context,
                    tuple_ty.get_field_types(lookup.context),
                    field_vals
                        .iter()
                        .map(|fv| fv.get_content(lookup.context).clone())
                        .collect(),
                );
                let c = Constant::unique(lookup.context, c);
                Some(c)
            })
        }
        ty::TyExpressionVariant::ArrayExplicit {
            elem_type,
            contents,
        } => {
            let (mut element_types, mut element_vals): (Vec<_>, Vec<_>) = (vec![], vec![]);
    
            for value in contents {
                let eval_expr_opt = const_eval_typed_expr(lookup, known_consts, value)?;
                if let Some(cv) = eval_expr_opt {
                    element_types.push(value.return_type);
                    element_vals.push(cv);
                } else {
                    return Err(ConstEvalError::CannotBeEvaluatedToConst {
                        span: expr.span.clone(),
                    });
                }
            }
    
            assert!(element_types.len() == contents.len());
            assert!(element_vals.len() == contents.len());
    
            create_array_from_vec(lookup, *elem_type, element_types, element_vals)
        }
        ty::TyExpressionVariant::ArrayRepeat {
            elem_type,
            value,
            length,
        } => {
            let constant = const_eval_typed_expr(lookup, known_consts, value)?.unwrap();
            let length = const_eval_typed_expr(lookup, known_consts, length)?
                .unwrap()
                .get_content(lookup.context)
                .as_uint()
                .unwrap() as usize;
            let element_vals = (0..length).map(|_| constant).collect::<Vec<_>>();
            let element_types = (0..length).map(|_| value.return_type).collect::<Vec<_>>();
    
            assert!(element_types.len() == length);
            assert!(element_vals.len() == length);
    
            create_array_from_vec(lookup, *elem_type, element_types, element_vals)
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
                lookup.engines,
                lookup.context,
                lookup.md_mgr,
                lookup.module,
                &enum_decl.variants,
            );
    
            if let Ok(enum_ty) = aggregate {
                let tag_value = ConstantContent::new_uint(lookup.context, 64, *tag as u64);
                let mut fields: Vec<ConstantContent> = vec![tag_value];
    
                match contents {
                    None => fields.push(ConstantContent::new_unit(lookup.context)),
                    Some(subexpr) => match const_eval_typed_expr(lookup, known_consts, subexpr)? {
                        Some(constant) => fields.push(constant.get_content(lookup.context).clone()),
                        None => {
                            return Err(ConstEvalError::CannotBeEvaluatedToConst {
                                span: variant_instantiation_span.clone(),
                            });
                        }
                    },
                }
    
                let fields_tys = enum_ty.get_field_types(lookup.context);
                let c = ConstantContent::new_struct(lookup.context, fields_tys, fields);
                let c = Constant::unique(lookup.context, c);
                Some(c)
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
        } => match const_eval_typed_expr(lookup, known_consts, prefix)?
            .map(|c| c.get_content(lookup.context).clone())
        {
            Some(ConstantContent {
                value: ConstantValue::Struct(fields),
                ..
            }) => {
                let field_kind = ty::ProjectionKind::StructField {
                    name: field_to_access.name.clone(),
                    field_to_access: Some(Box::new(field_to_access.clone())),
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
                .and_then(|field_idx| {
                    fields
                        .get(field_idx as usize)
                        .cloned()
                        .map(|c| Constant::unique(lookup.context, c))
                })
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
        } => match const_eval_typed_expr(lookup, known_consts, prefix)?
            .map(|c| c.get_content(lookup.context))
        {
            Some(ConstantContent {
                value: ConstantValue::Struct(fields),
                ..
            }) => fields
                .get(*elem_to_access_num)
                .cloned()
                .map(|c| Constant::unique(lookup.context, c)),
            _ => {
                return Err(ConstEvalError::CannotBeEvaluatedToConst {
                    span: expr.span.clone(),
                });
            }
        },
        ty::TyExpressionVariant::ImplicitReturn(e) => {
            if let Ok(Some(constant)) = const_eval_typed_expr(lookup, known_consts, e) {
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
        ty::TyExpressionVariant::Panic(exp) => {
            return Err(ConstEvalError::CannotBeEvaluatedToConst {
                span: exp.span.clone(),
            });
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
            match const_eval_typed_expr(lookup, known_consts, condition)?
                .map(|c| c.get_content(lookup.context))
            {
                Some(ConstantContent {
                    value: ConstantValue::Bool(cond),
                    ..
                }) => {
                    if *cond {
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
                    });
                }
            }
        }
        ty::TyExpressionVariant::CodeBlock(codeblock) => {
            const_eval_codeblock(lookup, known_consts, codeblock)?
        }
        ty::TyExpressionVariant::ArrayIndex { prefix, index } => {
            let prefix = const_eval_typed_expr(lookup, known_consts, prefix)?
                .map(|c| c.get_content(lookup.context).clone());
            let index = const_eval_typed_expr(lookup, known_consts, index)?
                .map(|c| c.get_content(lookup.context));
            match (prefix, index) {
                (
                    Some(ConstantContent {
                        value: ConstantValue::Array(items),
                        ..
                    }),
                    Some(ConstantContent {
                        value: ConstantValue::Uint(index),
                        ..
                    }),
                ) => {
                    let count = items.len() as u64;
                    if *index < count {
                        let c = Constant::unique(lookup.context, items[*index as usize].clone());
                        Some(c)
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
        ty::TyExpressionVariant::Ref(_) => {
            return Err(ConstEvalError::CompileError);
        }
        // We support *__elem_at(...)
        ty::TyExpressionVariant::Deref(expr) => {
            let value = expr
                .as_intrinsic()
                .filter(|x| matches!(x.kind, Intrinsic::ElemAt))
                .ok_or(ConstEvalError::CompileError)
                .and_then(|kind| {
                    const_eval_intrinsic(lookup, known_consts, kind)
                        .map(|c| c.map(|c| c.get_content(lookup.context).clone()))
                });
            if let Ok(Some(ConstantContent {
                value: ConstantValue::Reference(value),
                ..
            })) = value
            {
                let c = Constant::unique(lookup.context, *value.clone());
                Some(c)
            } else {
                return Err(ConstEvalError::CompileError);
            }
        }
        ty::TyExpressionVariant::EnumTag { exp } => {
            let value = const_eval_typed_expr(lookup, known_consts, exp)?
                .map(|x| x.get_content(lookup.context).value.clone());
            if let Some(ConstantValue::Struct(fields)) = value {
                Some(Constant::unique(lookup.context, fields[0].clone()))
            } else {
                return Err(ConstEvalError::CompileError);
            }
        }
        ty::TyExpressionVariant::UnsafeDowncast { exp, .. } => {
            let value = const_eval_typed_expr(lookup, known_consts, exp)?
                .map(|x| x.get_content(lookup.context).value.clone());
            if let Some(ConstantValue::Struct(fields)) = value {
                Some(Constant::unique(lookup.context, fields[1].clone()))
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
    
                let condition = const_eval_typed_expr(lookup, known_consts, condition)?;
                match condition.map(|x| x.get_content(lookup.context).value.clone()) {
                    Some(ConstantValue::Bool(true)) => {
                        // Break and continue are not implemented, so there is need for flow control here
                        let _ = const_eval_codeblock(lookup, known_consts, body)?;
                    }
                    _ => break,
                }
            }
    
            None
        }
        ty::TyExpressionVariant::Reassignment(r) => {
            let rhs = const_eval_typed_expr(lookup, known_consts, &r.rhs)?.unwrap();
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
                ty::TyReassignmentTarget::DerefAccess { .. } => {
                    return Err(ConstEvalError::CannotBeEvaluatedToConst {
                        span: expr.span.clone(),
                    });
                }
            }
        }
        ty::TyExpressionVariant::LazyOperator { op, lhs, rhs  } => {
            let lhs = const_eval_typed_expr(lookup, known_consts, lhs)?.unwrap();
            match (lhs.get_content(lookup.context).as_bool().unwrap(), op) {
                (true, LazyOp::And) | (false, LazyOp::Or) => {
                    const_eval_typed_expr(lookup, known_consts, rhs)?
                },
                (false, LazyOp::And) | (true, LazyOp::Or) => Some(lhs),
            }
        }
        ty::TyExpressionVariant::FunctionParameter
        | ty::TyExpressionVariant::AsmExpression { .. }
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
                        span: decl.span(lookup.engines).clone(),
                    })
                }
            }
            ty::TyAstNodeContent::Declaration(ty::TyDecl::ConstantDecl(const_decl)) => {
                let ty_const_decl = lookup.engines.de().get_constant(&const_decl.decl_id);
                if let Some(constant) = ty_const_decl
                    .value
                    .as_ref()
                    .filter(|expr| !contains_outer_vars(expr, &HashSet::new()))
                    .and_then(|expr| const_eval_typed_expr(lookup, known_consts, expr).ok())
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
                    if let Ok(Some(constant)) = const_eval_typed_expr(lookup, known_consts, e) {
                        Ok(Some(constant))
                    } else {
                        Err(ConstEvalError::CannotBeEvaluatedToConst {
                            span: e.span.clone(),
                        })
                    }
                }
                _ => {
                    if const_eval_typed_expr(lookup, known_consts, e).is_err() {
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
            ty::TyAstNodeContent::Error(_, _) => Err(ConstEvalError::CannotBeEvaluatedToConst {
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

fn as_encode_buffer<'a>(context: &'a Context, buffer: &Constant) -> Option<(&'a Vec<u8>, u64)> {
    match &buffer.get_content(context).value {
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
    let c = ConstantContent {
        ty: Type::new_struct(
            lookup.context,
            vec![
                Type::get_slice(lookup.context),
                Type::get_uint64(lookup.context),
            ],
        ),
        value: ConstantValue::Struct(vec![
            ConstantContent {
                ty: Type::get_slice(lookup.context),
                value: ConstantValue::RawUntypedSlice(bytes),
            },
            ConstantContent {
                ty: Type::get_uint64(lookup.context),
                value: ConstantValue::Uint(len),
            },
        ]),
    };
    Constant::unique(lookup.context, c)
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
        Intrinsic::Add | Intrinsic::Sub | Intrinsic::Mul | Intrinsic::Div | Intrinsic::Mod => {
            let ty = args[0].get_content(lookup.context).ty;
            assert!(
                args.len() == 2 && ty.eq(lookup.context, &args[1].get_content(lookup.context).ty)
            );

            use ConstantValue::*;
            let c = match (
                &args[0].get_content(lookup.context).value,
                &args[1].get_content(lookup.context).value,
            ) {
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
                        Some(result) => Ok(Some(ConstantContent {
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
                        Some(result) => Ok(Some(ConstantContent {
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
            };
            c.map(|c| c.map(|c| Constant::unique(lookup.context, c)))
        }
        Intrinsic::And | Intrinsic::Or | Intrinsic::Xor => {
            let ty = args[0].get_content(lookup.context).ty;
            assert!(
                args.len() == 2 && ty.eq(lookup.context, &args[1].get_content(lookup.context).ty)
            );

            use ConstantValue::*;
            let c = match (
                &args[0].get_content(lookup.context).value,
                &args[1].get_content(lookup.context).value,
            ) {
                (Uint(arg1), Uint(ref arg2)) => {
                    // All arithmetic is done as if it were u64
                    let result = match intrinsic.kind {
                        Intrinsic::And => Some(arg1.bitand(arg2)),
                        Intrinsic::Or => Some(arg1.bitor(*arg2)),
                        Intrinsic::Xor => Some(arg1.bitxor(*arg2)),
                        _ => unreachable!(),
                    };

                    match result {
                        Some(sum) => Ok(Some(ConstantContent {
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
                        Some(sum) => Ok(Some(ConstantContent {
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
                        Some(result) => Ok(Some(ConstantContent {
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
            };
            c.map(|c| c.map(|c| Constant::unique(lookup.context, c)))
        }
        Intrinsic::Lsh | Intrinsic::Rsh => {
            assert!(args.len() == 2);
            assert!(
                args[0]
                    .get_content(lookup.context)
                    .ty
                    .is_uint(lookup.context)
                    || args[0]
                        .get_content(lookup.context)
                        .ty
                        .is_b256(lookup.context)
            );
            assert!(args[1]
                .get_content(lookup.context)
                .ty
                .is_uint64(lookup.context));

            let ty = args[0].get_content(lookup.context).ty;

            use ConstantValue::*;
            let c = match (
                &args[0].get_content(lookup.context).value,
                &args[1].get_content(lookup.context).value,
            ) {
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
                        Some(sum) => Ok(Some(ConstantContent {
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
                        Some(value) => Ok(Some(ConstantContent {
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
                        Some(result) => Ok(Some(ConstantContent {
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
            };
            c.map(|c| c.map(|c| Constant::unique(lookup.context, c)))
        }
        Intrinsic::SizeOfType => {
            let targ = &intrinsic.type_arguments[0];
            let ir_type = convert_resolved_type_id(
                lookup.engines,
                lookup.context,
                lookup.md_mgr,
                lookup.module,
                lookup.function_compiler,
                targ.type_id(),
                &targ.span(),
            )
            .map_err(|_| ConstEvalError::CompileError)?;
            let c = ConstantContent {
                ty: Type::get_uint64(lookup.context),
                value: ConstantValue::Uint(ir_type.size(lookup.context).in_bytes()),
            };

            Ok(Some(Constant::unique(lookup.context, c)))
        }
        Intrinsic::SizeOfVal => {
            let val = &intrinsic.arguments[0];
            let type_id = val.return_type;
            let ir_type = convert_resolved_type_id(
                lookup.engines,
                lookup.context,
                lookup.md_mgr,
                lookup.module,
                lookup.function_compiler,
                type_id,
                &val.span,
            )
            .map_err(|_| ConstEvalError::CompileError)?;
            let c = ConstantContent {
                ty: Type::get_uint64(lookup.context),
                value: ConstantValue::Uint(ir_type.size(lookup.context).in_bytes()),
            };
            Ok(Some(Constant::unique(lookup.context, c)))
        }
        Intrinsic::SizeOfStr => {
            let targ = &intrinsic.type_arguments[0];
            let ir_type = convert_resolved_type_id(
                lookup.engines,
                lookup.context,
                lookup.md_mgr,
                lookup.module,
                lookup.function_compiler,
                targ.type_id(),
                &targ.span(),
            )
            .map_err(|_| ConstEvalError::CompileError)?;
            let c = ConstantContent {
                ty: Type::get_uint64(lookup.context),
                value: ConstantValue::Uint(
                    ir_type.get_string_len(lookup.context).unwrap_or_default(),
                ),
            };
            Ok(Some(Constant::unique(lookup.context, c)))
        }
        Intrinsic::AssertIsStrArray => {
            let targ = &intrinsic.type_arguments[0];
            let ir_type = convert_resolved_type_id(
                lookup.engines,
                lookup.context,
                lookup.md_mgr,
                lookup.module,
                lookup.function_compiler,
                targ.type_id(),
                &targ.span(),
            )
            .map_err(|_| ConstEvalError::CompileError)?;
            match ir_type.get_content(lookup.context) {
                TypeContent::StringSlice | TypeContent::StringArray(_) => {
                    let c = ConstantContent {
                        ty: Type::get_unit(lookup.context),
                        value: ConstantValue::Unit,
                    };
                    Ok(Some(Constant::unique(lookup.context, c)))
                }
                _ => Err(ConstEvalError::CompileError),
            }
        }
        Intrinsic::ToStrArray => {
            assert!(args.len() == 1);
            match &args[0].get_content(lookup.context).value {
                ConstantValue::String(s) => {
                    let c = ConstantContent::new_string(lookup.context, s.to_vec());
                    Ok(Some(Constant::unique(lookup.context, c)))
                }
                _ => {
                    unreachable!("Type checker allowed non string value for ToStrArray")
                }
            }
        }
        Intrinsic::Eq => {
            assert!(args.len() == 2);
            let c = ConstantContent {
                ty: Type::get_bool(lookup.context),
                value: ConstantValue::Bool(args[0] == args[1]),
            };
            Ok(Some(Constant::unique(lookup.context, c)))
        }
        Intrinsic::Gt => match (
            &args[0].get_content(lookup.context).value,
            &args[1].get_content(lookup.context).value,
        ) {
            (ConstantValue::Uint(val1), ConstantValue::Uint(val2)) => {
                let c = ConstantContent {
                    ty: Type::get_bool(lookup.context),
                    value: ConstantValue::Bool(val1 > val2),
                };
                Ok(Some(Constant::unique(lookup.context, c)))
            }
            (ConstantValue::U256(val1), ConstantValue::U256(val2)) => {
                let c = ConstantContent {
                    ty: Type::get_bool(lookup.context),
                    value: ConstantValue::Bool(val1 > val2),
                };
                Ok(Some(Constant::unique(lookup.context, c)))
            }
            _ => {
                unreachable!("Type checker allowed non integer value for GreaterThan")
            }
        },
        Intrinsic::Lt => match (
            &args[0].get_content(lookup.context).value,
            &args[1].get_content(lookup.context).value,
        ) {
            (ConstantValue::Uint(val1), ConstantValue::Uint(val2)) => {
                let c = ConstantContent {
                    ty: Type::get_bool(lookup.context),
                    value: ConstantValue::Bool(val1 < val2),
                };
                Ok(Some(Constant::unique(lookup.context, c)))
            }
            (ConstantValue::U256(val1), ConstantValue::U256(val2)) => {
                let c = ConstantContent {
                    ty: Type::get_bool(lookup.context),
                    value: ConstantValue::Bool(val1 < val2),
                };
                Ok(Some(Constant::unique(lookup.context, c)))
            }
            _ => {
                unreachable!("Type checker allowed non integer value for LessThan")
            }
        },
        Intrinsic::AddrOf
        | Intrinsic::Alloc
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
            assert!(
                args[0]
                    .get_content(lookup.context)
                    .ty
                    .is_uint(lookup.context)
                    || args[0]
                        .get_content(lookup.context)
                        .ty
                        .is_b256(lookup.context)
            );

            let Some(arg) = args.into_iter().next() else {
                unreachable!("Unexpected 'not' without any arguments");
            };

            let c = match &arg.get_content(lookup.context).value {
                ConstantValue::Uint(n) => {
                    let n = match arg
                        .get_content(lookup.context)
                        .ty
                        .get_uint_width(lookup.context)
                    {
                        Some(8) => !(*n as u8) as u64,
                        Some(16) => !(*n as u16) as u64,
                        Some(32) => !(*n as u32) as u64,
                        Some(64) => !n,
                        _ => unreachable!("Invalid unsigned integer width"),
                    };
                    Ok(Some(ConstantContent {
                        ty: arg.get_content(lookup.context).ty,
                        value: ConstantValue::Uint(n),
                    }))
                }
                ConstantValue::U256(n) => Ok(Some(ConstantContent {
                    ty: arg.get_content(lookup.context).ty,
                    value: ConstantValue::U256(n.not()),
                })),
                ConstantValue::B256(v) => Ok(Some(ConstantContent {
                    ty: arg.get_content(lookup.context).ty,
                    value: ConstantValue::B256(v.not()),
                })),
                _ => {
                    unreachable!("Type checker allowed non integer value for Not");
                }
            };
            c.map(|c| c.map(|c| Constant::unique(lookup.context, c)))
        }
        Intrinsic::ContractCall | Intrinsic::ContractRet => {
            Err(ConstEvalError::CannotBeEvaluatedToConst {
                span: intrinsic.span.clone(),
            })
        }
        Intrinsic::EncodeBufferEmpty => Ok(Some(to_encode_buffer(lookup, vec![], 0))),
        Intrinsic::EncodeBufferAppend => {
            assert!(args.len() == 2);

            let (slice, mut len) = as_encode_buffer(lookup.context, &args[0]).unwrap();
            let mut bytes = slice.clone();

            use ConstantValue::*;
            match &args[1].get_content(lookup.context).value {
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

            let (slice, len) = as_encode_buffer(lookup.context, &args[0]).unwrap();
            let bytes = slice.clone();

            let c = ConstantContent {
                ty: Type::get_slice(lookup.context),
                value: ConstantValue::RawUntypedSlice(bytes[0..(len as usize)].to_vec()),
            };
            Ok(Some(Constant::unique(lookup.context, c)))
        }
        Intrinsic::Slice => {
            let start = args[1]
                .get_content(lookup.context)
                .as_uint()
                .expect("Type check allowed non u64") as usize;
            let end = args[2]
                .get_content(lookup.context)
                .as_uint()
                .expect("Type check allowed non u64") as usize;

            match &args[0].get_content(lookup.context).value {
                ConstantValue::Array(elements) => {
                    let slice = elements
                        .get(start..end)
                        .ok_or(ConstEvalError::CompileError)?;
                    let elem_type = args[0]
                        .get_content(lookup.context)
                        .ty
                        .get_array_elem_type(lookup.context)
                        .expect("unexpected non array");
                    let s = slice.to_vec();
                    let c = ConstantContent {
                        ty: Type::get_typed_slice(lookup.context, elem_type),
                        value: ConstantValue::Slice(s),
                    };
                    Ok(Some(Constant::unique(lookup.context, c)))
                }
                ConstantValue::Reference(r) => match &r.value {
                    ConstantValue::Slice(elements) => {
                        let slice = elements
                            .get(start..end)
                            .ok_or(ConstEvalError::CompileError)?;
                        let elem_type = args[0]
                            .get_content(lookup.context)
                            .ty
                            .get_typed_slice_elem_type(lookup.context)
                            .expect("unexpected non slice");
                        let s = slice.to_vec();
                        let c = ConstantContent {
                            ty: Type::get_typed_slice(lookup.context, elem_type),
                            value: ConstantValue::Slice(s),
                        };
                        Ok(Some(Constant::unique(lookup.context, c)))
                    }
                    _ => Err(ConstEvalError::CannotBeEvaluatedToConst {
                        span: intrinsic.span.clone(),
                    }),
                },
                _ => Err(ConstEvalError::CannotBeEvaluatedToConst {
                    span: intrinsic.span.clone(),
                }),
            }
        }
        Intrinsic::ElemAt => {
            let idx = args[1]
                .get_content(lookup.context)
                .as_uint()
                .expect("Type check allowed non u64") as usize;

            match &args[0].get_content(lookup.context).value {
                ConstantValue::Reference(r) => match &r.value {
                    ConstantValue::Slice(elements) => {
                        let v = elements[idx].clone();
                        let c = ConstantContent {
                            ty: Type::new_typed_pointer(lookup.context, v.ty),
                            value: ConstantValue::Reference(Box::new(v)),
                        };
                        Ok(Some(Constant::unique(lookup.context, c)))
                    }
                    _ => Err(ConstEvalError::CannotBeEvaluatedToConst {
                        span: intrinsic.span.clone(),
                    }),
                },
                _ => Err(ConstEvalError::CannotBeEvaluatedToConst {
                    span: intrinsic.span.clone(),
                }),
            }
        }
        Intrinsic::Transmute => {
            let src_type = &intrinsic.type_arguments[0];
            let src_ir_type = convert_resolved_type_id(
                lookup.engines,
                lookup.context,
                lookup.md_mgr,
                lookup.module,
                lookup.function_compiler,
                src_type.type_id(),
                &src_type.span(),
            )
            .unwrap();

            let dst_type = &intrinsic.type_arguments[1];
            let dst_ir_type = convert_resolved_type_id(
                lookup.engines,
                lookup.context,
                lookup.md_mgr,
                lookup.module,
                lookup.function_compiler,
                dst_type.type_id(),
                &dst_type.span(),
            )
            .unwrap();

            // check IR sizes match
            let src_ir_type_in_bytes = src_ir_type.size(lookup.context).in_bytes();
            let dst_ir_type_in_bytes = dst_ir_type.size(lookup.context).in_bytes();
            if src_ir_type_in_bytes != dst_ir_type_in_bytes {
                return Err(ConstEvalError::CompileError);
            }

            fn append_bytes(
                ctx: &Context<'_>,
                bytes: &mut Vec<u8>,
                t: &Type,
                value: &ConstantValue,
            ) -> Result<(), ConstEvalError> {
                match t.get_content(ctx) {
                    TypeContent::Struct(fields) => match value {
                        ConstantValue::Struct(constants) => {
                            for (field_type, field) in fields.iter().zip(constants.iter()) {
                                append_bytes(ctx, bytes, field_type, &field.value)?;
                            }
                        }
                        _ => unreachable!(),
                    },
                    TypeContent::Array(item_type, size) => match value {
                        ConstantValue::Array(items) => {
                            assert!(*size as usize == items.len());
                            for item in items {
                                append_bytes(ctx, bytes, item_type, &item.value)?;
                            }
                        }
                        _ => unreachable!(),
                    },
                    TypeContent::Uint(8) => match value {
                        ConstantValue::Uint(v) => {
                            bytes.extend((*v as u8).to_be_bytes());
                        }
                        _ => unreachable!(),
                    },
                    TypeContent::Uint(16) => match value {
                        ConstantValue::Uint(v) => {
                            bytes.extend([0u8, 0u8, 0u8, 0u8, 0u8, 0u8]);
                            bytes.extend((*v as u16).to_be_bytes());
                        }
                        _ => unreachable!(),
                    },
                    TypeContent::Uint(32) => match value {
                        ConstantValue::Uint(v) => {
                            bytes.extend([0u8, 0u8, 0u8, 0u8]);
                            bytes.extend((*v as u32).to_be_bytes());
                        }
                        _ => unreachable!(),
                    },
                    TypeContent::Uint(64) => match value {
                        ConstantValue::Uint(v) => {
                            bytes.extend((*v).to_be_bytes());
                        }
                        _ => unreachable!(),
                    },
                    _ => return Err(ConstEvalError::CompileError),
                }
                Ok(())
            }

            fn transmute_bytes(
                ctx: &Context<'_>,
                bytes: &mut std::io::Cursor<Vec<u8>>,
                t: &Type,
            ) -> Result<ConstantContent, ConstEvalError> {
                Ok(match t.get_content(ctx) {
                    TypeContent::Uint(8) => {
                        let mut buffer = [0u8];
                        let _ = bytes.read_exact(&mut buffer);
                        ConstantContent {
                            ty: Type::get_uint8(ctx),
                            value: ConstantValue::Uint(buffer[0] as u64),
                        }
                    }
                    TypeContent::Uint(16) => {
                        let mut buffer = [0u8; 8]; // u16 = u64 at runtime
                        let _ = bytes.read_exact(&mut buffer);
                        let buffer = [buffer[6], buffer[7]];
                        ConstantContent {
                            ty: Type::get_uint16(ctx),
                            value: ConstantValue::Uint(u16::from_be_bytes(buffer) as u64),
                        }
                    }
                    TypeContent::Uint(32) => {
                        let mut buffer = [0u8; 8]; // u32 = u64 at runtime
                        let _ = bytes.read_exact(&mut buffer);
                        let buffer = [buffer[4], buffer[5], buffer[6], buffer[7]];
                        ConstantContent {
                            ty: Type::get_uint32(ctx),
                            value: ConstantValue::Uint(u32::from_be_bytes(buffer) as u64),
                        }
                    }
                    TypeContent::Uint(64) => {
                        let mut buffer = [0u8; 8];
                        let _ = bytes.read_exact(&mut buffer);
                        ConstantContent {
                            ty: Type::get_uint64(ctx),
                            value: ConstantValue::Uint(u64::from_be_bytes(buffer)),
                        }
                    }
                    _ => return Err(ConstEvalError::CompileError),
                })
            }

            let mut runtime_bytes = vec![];
            append_bytes(
                lookup.context,
                &mut runtime_bytes,
                &src_ir_type,
                &args[0].get_content(lookup.context).value,
            )?;
            let mut cursor = std::io::Cursor::new(runtime_bytes);
            let c = transmute_bytes(lookup.context, &mut cursor, &dst_ir_type)?;
            Ok(Some(Constant::unique(lookup.context, c)))
        }
        Intrinsic::Dbg => {
            unreachable!("__dbg should not exist in the typed tree")
        }
        Intrinsic::RuntimeMemoryId => {
            assert!(intrinsic.type_arguments.len() == 1);
            assert!(intrinsic.arguments.is_empty());

            let t = &intrinsic.type_arguments[0];
            let t = convert_resolved_type_id(
                lookup.engines,
                lookup.context,
                lookup.md_mgr,
                lookup.module,
                lookup.function_compiler,
                t.type_id(),
                &t.span(),
            )
            .unwrap();

            let id = get_memory_id(lookup.context, t);
            let c = ConstantContent {
                ty: Type::get_uint64(lookup.context),
                value: ConstantValue::Uint(id),
            };

            Ok(Some(Constant::unique(lookup.context, c)))
        }
        Intrinsic::EncodingMemoryId => {
            assert!(intrinsic.type_arguments.len() == 1);
            assert!(intrinsic.arguments.is_empty());

            let t = intrinsic.type_arguments[0].as_type_argument().unwrap();

            let id = get_encoding_id(lookup.engines, t.type_id);
            let c = ConstantContent {
                ty: Type::get_uint64(lookup.context),
                value: ConstantValue::Uint(id),
            };

            Ok(Some(Constant::unique(lookup.context, c)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sway_error::handler::Handler;
    use sway_features::ExperimentalFeatures;
    use sway_ir::{Backtrace, Kind};
    use sway_types::ProgramId;

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
            ExperimentalFeatures::default(),
            Backtrace::default(),
        );
        let mut md_mgr = MetadataManager::default();
        let core_lib = namespace::Package::new(
            sway_types::Ident::new_no_span("assert_is_constant_test".to_string()),
            None,
            ProgramId::new(0),
            false,
        );

        let r = crate::compile_to_ast(
            &handler,
            &engines,
            format!("library; {prefix} fn f() -> u64 {{ {expr}; 0 }}")
                .as_str()
                .into(),
            core_lib,
            None,
            "test",
            None,
            ExperimentalFeatures::default(),
        );

        let (errors, _warnings, _infos) = handler.consume();

        if !errors.is_empty() {
            panic!("{errors:#?}");
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
