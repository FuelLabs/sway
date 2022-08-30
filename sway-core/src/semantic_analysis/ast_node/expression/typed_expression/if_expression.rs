use sway_types::Span;

use crate::{error::*, semantic_analysis::*, type_system::*, types::DeterministicallyAborts};

use super::TypedExpression;

pub(crate) fn instantiate_if_expression(
    ctx: TypeCheckContext,
    condition: TypedExpression,
    then: TypedExpression,
    r#else: Option<TypedExpression>,
    span: Span,
) -> CompileResult<TypedExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];

    // if the branch aborts, then its return type doesn't matter.
    let then_deterministically_aborts = then.deterministically_aborts();
    if !then_deterministically_aborts {
        // if this does not deterministically_abort, check the block return type
        let ty_to_check = if r#else.is_some() {
            ctx.type_annotation()
        } else {
            insert_type(TypeInfo::Tuple(vec![]))
        };
        let (mut new_warnings, new_errors) = unify_with_self(
            then.return_type,
            ty_to_check,
            &ctx.declaration_engine,
            ctx.self_type(),
            &then.span,
            "`then` branch must return expected type.",
        );
        warnings.append(&mut new_warnings);
        errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
    }
    let mut else_deterministically_aborts = false;
    let r#else = r#else.map(|r#else| {
        else_deterministically_aborts = r#else.deterministically_aborts();
        let ty_to_check = if then_deterministically_aborts {
            ctx.type_annotation()
        } else {
            then.return_type
        };
        if !else_deterministically_aborts {
            // if this does not deterministically_abort, check the block return type
            let (mut new_warnings, new_errors) = unify_with_self(
                r#else.return_type,
                ty_to_check,
                &ctx.declaration_engine,
                ctx.self_type(),
                &r#else.span,
                "`else` branch must return expected type.",
            );
            warnings.append(&mut new_warnings);
            errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
        }
        Box::new(r#else)
    });

    let r#else_ret_ty = r#else
        .as_ref()
        .map(|x| x.return_type)
        .unwrap_or_else(|| insert_type(TypeInfo::Tuple(Vec::new())));
    // if there is a type annotation, then the else branch must exist
    if !else_deterministically_aborts && !then_deterministically_aborts {
        let (mut new_warnings, new_errors) = unify_with_self(
            then.return_type,
            r#else_ret_ty,
            &ctx.declaration_engine,
            ctx.self_type(),
            &span,
            "The two branches of an if expression must return the same type.",
        );
        warnings.append(&mut new_warnings);
        if new_errors.is_empty() {
            if !look_up_type_id(r#else_ret_ty).is_unit() && r#else.is_none() {
                errors.push(CompileError::NoElseBranch {
                    span: span.clone(),
                    r#type: look_up_type_id(ctx.type_annotation()).to_string(),
                });
            }
        } else {
            errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
        }
    }

    let return_type = if !then_deterministically_aborts {
        then.return_type
    } else {
        r#else_ret_ty
    };
    let exp = TypedExpression {
        expression: TypedExpressionVariant::IfExp {
            condition: Box::new(condition),
            then: Box::new(then),
            r#else,
        },
        is_constant: IsConstant::No,
        return_type,
        span,
    };
    ok(exp, warnings, errors)
}
