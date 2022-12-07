use sway_error::error::CompileError;
use sway_types::Span;

use crate::{
    error::*, language::ty, semantic_analysis::TypeCheckContext, type_system::*,
    types::DeterministicallyAborts,
};

pub(crate) fn instantiate_if_expression(
    ctx: TypeCheckContext,
    condition: ty::TyExpression,
    then: ty::TyExpression,
    r#else: Option<ty::TyExpression>,
    span: Span,
) -> CompileResult<ty::TyExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let type_engine = ctx.type_engine;
    let declaration_engine = ctx.declaration_engine;
    let engines = ctx.engines();

    // if the branch aborts, then its return type doesn't matter.
    let then_deterministically_aborts = then.deterministically_aborts(declaration_engine, true);
    if !then_deterministically_aborts {
        // if this does not deterministically_abort, check the block return type
        let ty_to_check = if r#else.is_some() {
            ctx.type_annotation()
        } else {
            type_engine.insert_type(TypeInfo::Tuple(vec![]))
        };
        append!(
            type_engine.unify_with_self(
                ctx.declaration_engine,
                then.return_type,
                ty_to_check,
                ctx.self_type(),
                &then.span,
                "`then` branch must return expected type.",
            ),
            warnings,
            errors
        );
    }
    let mut else_deterministically_aborts = false;
    let r#else = r#else.map(|r#else| {
        else_deterministically_aborts = r#else.deterministically_aborts(declaration_engine, true);
        let ty_to_check = if then_deterministically_aborts {
            ctx.type_annotation()
        } else {
            then.return_type
        };
        if !else_deterministically_aborts {
            // if this does not deterministically_abort, check the block return type
            append!(
                type_engine.unify_with_self(
                    ctx.declaration_engine,
                    r#else.return_type,
                    ty_to_check,
                    ctx.self_type(),
                    &r#else.span,
                    "`else` branch must return expected type.",
                ),
                warnings,
                errors
            );
        }
        Box::new(r#else)
    });

    let r#else_ret_ty = r#else
        .as_ref()
        .map(|x| x.return_type)
        .unwrap_or_else(|| type_engine.insert_type(TypeInfo::Tuple(Vec::new())));
    // if there is a type annotation, then the else branch must exist
    if !else_deterministically_aborts && !then_deterministically_aborts {
        let (mut new_warnings, mut new_errors) = type_engine.unify_with_self(
            ctx.declaration_engine,
            then.return_type,
            r#else_ret_ty,
            ctx.self_type(),
            &span,
            "The two branches of an if expression must return the same type.",
        );
        warnings.append(&mut new_warnings);
        if new_errors.is_empty() {
            if !type_engine.look_up_type_id(r#else_ret_ty).is_unit() && r#else.is_none() {
                errors.push(CompileError::NoElseBranch {
                    span: span.clone(),
                    r#type: engines.help_out(ctx.type_annotation()).to_string(),
                });
            }
        } else {
            errors.append(&mut new_errors);
        }
    }

    let return_type = if !then_deterministically_aborts {
        then.return_type
    } else {
        r#else_ret_ty
    };
    let exp = ty::TyExpression {
        expression: ty::TyExpressionVariant::IfExp {
            condition: Box::new(condition),
            then: Box::new(then),
            r#else,
        },
        return_type,
        span,
    };
    ok(exp, warnings, errors)
}
