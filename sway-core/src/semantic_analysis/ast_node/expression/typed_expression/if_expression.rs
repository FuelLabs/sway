use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::Span;

use crate::{
    language::ty, semantic_analysis::TypeCheckContext, type_system::*,
    types::DeterministicallyAborts,
};

pub(crate) fn instantiate_if_expression(
    handler: &Handler,
    ctx: TypeCheckContext,
    condition: ty::TyExpression,
    then: ty::TyExpression,
    r#else: Option<ty::TyExpression>,
    span: Span,
) -> Result<ty::TyExpression, ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let decl_engine = ctx.engines.de();
    let engines = ctx.engines();

    // if the branch aborts, then its return type doesn't matter.
    let then_deterministically_aborts = then.deterministically_aborts(decl_engine, true);
    if !then_deterministically_aborts {
        // if this does not deterministically_abort, check the block return type
        let ty_to_check = if r#else.is_some() {
            ctx.type_annotation()
        } else {
            type_engine.insert(engines, TypeInfo::Tuple(vec![]), then.span.source_id())
        };
        type_engine.unify(
            handler,
            engines,
            then.return_type,
            ty_to_check,
            &then.span,
            "`then` branch must return expected type.",
            None,
        );
    }
    let mut else_deterministically_aborts = false;
    let r#else = r#else.map(|r#else| {
        else_deterministically_aborts = r#else.deterministically_aborts(decl_engine, true);
        let ty_to_check = if then_deterministically_aborts {
            ctx.type_annotation()
        } else {
            then.return_type
        };
        if !else_deterministically_aborts {
            // if this does not deterministically_abort, check the block return type
            type_engine.unify(
                handler,
                engines,
                r#else.return_type,
                ty_to_check,
                &r#else.span,
                "`else` branch must return expected type.",
                None,
            );
        }
        Box::new(r#else)
    });

    let r#else_ret_ty = r#else
        .as_ref()
        .map(|x| x.return_type)
        .unwrap_or_else(|| type_engine.insert(engines, TypeInfo::Tuple(Vec::new()), span.source_id()));
    // if there is a type annotation, then the else branch must exist
    if !else_deterministically_aborts && !then_deterministically_aborts {
        // delay emitting the errors until we decide if this is a missing else branch or some other set of errors
        let h = Handler::default();
        type_engine.unify(
            &h,
            engines,
            then.return_type,
            r#else_ret_ty,
            &span,
            "The two branches of an if expression must return the same type.",
            None,
        );

        let (new_errors, new_warnings) = h.consume();
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
    Ok(exp)
}
