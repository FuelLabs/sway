use sway_types::Span;

use crate::{error::*, semantic_analysis::*, type_engine::*, types::DeterministicallyAborts};

use super::TypedExpression;

pub(crate) fn instantiate_if_expression(
    condition: TypedExpression,
    then: TypedExpression,
    r#else: Option<TypedExpression>,
    span: Span,
    type_annotation: TypeId,
    self_type: TypeId,
) -> CompileResult<TypedExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];

    // if the branch aborts, then its return type doesn't matter.
    let then_deterministically_aborts = then.deterministically_aborts();
    if !then_deterministically_aborts {
        // if this does not deterministically_abort, check the block return type
        let ty_to_check = if r#else.is_some() {
            type_annotation
        } else {
            insert_type(TypeInfo::Tuple(vec![]))
        };
        let (mut new_warnings, new_errors) = unify_with_self(
            then.return_type,
            ty_to_check,
            self_type,
            &then.span,
            "`then` branch must return expected type.",
        );
        warnings.append(&mut new_warnings);
        errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
    }

    let mut else_deterministically_aborts = false;
    let mut else_return_type = insert_type(TypeInfo::Tuple(Vec::new()));
    if let Some(ref r#else) = r#else {
        else_deterministically_aborts = r#else.deterministically_aborts();
        if !else_deterministically_aborts {
            // if this does not deterministically_abort, check the block return type
            let (mut new_warnings, new_errors) = unify_with_self(
                r#else.return_type,
                type_annotation,
                self_type,
                &r#else.span,
                "`else` branch must return expected type.",
            );
            warnings.append(&mut new_warnings);
            errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
        }
        else_return_type = r#else.return_type;
    }

    // if there is a type annotation, then the else branch must exist
    if !else_deterministically_aborts && !then_deterministically_aborts {
        let (mut new_warnings, new_errors) = unify_with_self(
            then.return_type,
            else_return_type,
            self_type,
            &span,
            "The two branches of an if expression must return the same type.",
        );
        warnings.append(&mut new_warnings);
        if new_errors.is_empty() {
            if !look_up_type_id(then.return_type).is_unit() && r#else.is_none() {
                errors.push(CompileError::NoElseBranch {
                    span: span.clone(),
                    r#type: look_up_type_id(type_annotation).to_string(),
                });
            }
        } else {
            errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
        }
    }

    let return_type = then.return_type;
    let exp = TypedExpression {
        expression: TypedExpressionVariant::IfExp {
            condition: Box::new(condition),
            then: Box::new(then),
            r#else: r#else.map(Box::new),
        },
        is_constant: IsConstant::No,
        return_type,
        span,
    };
    ok(exp, warnings, errors)
}
