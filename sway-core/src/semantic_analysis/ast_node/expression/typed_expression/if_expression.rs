use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
    type_error::TypeError,
};
use sway_types::Span;

use crate::{language::ty, semantic_analysis::TypeCheckContext, type_system::*};

pub(crate) fn instantiate_if_expression(
    handler: &Handler,
    ctx: TypeCheckContext,
    condition: ty::TyExpression,
    then: ty::TyExpression,
    r#else: Option<ty::TyExpression>,
    span: Span,
) -> Result<ty::TyExpression, ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    // Check the then block return type
    let ty_to_check = if r#else.is_some() {
        ctx.type_annotation()
    } else {
        type_engine.id_of_unit()
    };

    // We check then_type_is_never and else_type_is_never before unifying to make sure we don't
    // unify ty_to_check with Never when another branch is not Never.
    let then_type_is_never = matches!(*type_engine.get(then.return_type), TypeInfo::Never);
    let else_type_is_never = r#else.is_some()
        && matches!(
            *type_engine.get(r#else.as_ref().unwrap().return_type),
            TypeInfo::Never
        );

    if r#else.is_none() || !then_type_is_never || else_type_is_never {
        type_engine.unify(
            handler,
            engines,
            then.return_type,
            ty_to_check,
            &then.span,
            "`then` branch must return expected type.",
            || None,
        );
    }

    let r#else = r#else.map(|r#else| {
        if !else_type_is_never || then_type_is_never {
            // Check the else block return type
            type_engine.unify(
                handler,
                engines,
                r#else.return_type,
                ty_to_check,
                &r#else.span,
                "`else` branch must return expected type.",
                || None,
            );
        }
        Box::new(r#else)
    });

    let r#else_ret_ty = r#else
        .as_ref()
        .map(|x| x.return_type)
        .unwrap_or_else(|| type_engine.id_of_unit());

    // delay emitting the errors until we decide if this is a missing else branch or some other set of errors
    let h = Handler::default();

    let unify_check = UnifyCheck::coercion(engines);
    // Perform unify check in both ways as Never coercion is not commutative
    if !unify_check.check(then.return_type, r#else_ret_ty)
        && !unify_check.check(r#else_ret_ty, then.return_type)
    {
        h.emit_err(CompileError::TypeError(TypeError::MismatchedType {
            expected: engines.help_out(then.return_type).to_string(),
            received: engines.help_out(r#else_ret_ty).to_string(),
            help_text: "The two branches of an if expression must return the same type."
                .to_string(),
            span: span.clone(),
        }));
    }

    let (new_errors, new_warnings, new_infos) = h.consume();
    for info in new_infos {
        handler.emit_info(info);
    }
    for warn in new_warnings {
        handler.emit_warn(warn);
    }
    if new_errors.is_empty() {
        if !type_engine.get(r#else_ret_ty).is_unit() && r#else.is_none() {
            handler.emit_err(CompileError::NoElseBranch {
                span: span.clone(),
                r#type: engines.help_out(ctx.type_annotation()).to_string(),
            });
        }
    } else {
        for err in new_errors {
            handler.emit_err(err);
        }
    }

    let return_type = if !matches!(*type_engine.get(then.return_type), TypeInfo::Never) {
        then.return_type
    } else {
        r#else_ret_ty
    };
    let exp = ty::TyExpression {
        expression: ty::TyExpressionVariant::IfExp {
            condition: Box::new(condition),
            then: Box::new(then.clone()),
            r#else,
        },
        return_type,
        span,
    };
    Ok(exp)
}
