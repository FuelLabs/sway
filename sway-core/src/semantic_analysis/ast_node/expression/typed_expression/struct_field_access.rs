use sway_types::{Ident, Span};

use crate::{
    error::{err, ok},
    semantic_analysis::{IsConstant, TypedExpression, TypedExpressionVariant, TypedStructField},
    CompileError, CompileResult, NamespaceRef, NamespaceWrapper,
};

pub(crate) fn instantiate_struct_field_access(
    parent: TypedExpression,
    field_to_access: Ident,
    span: Span,
    namespace: NamespaceRef,
) -> CompileResult<TypedExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let (fields, struct_name) = check!(
        namespace.get_struct_type_fields(parent.return_type, parent.span.as_str(), &parent.span),
        return err(warnings, errors),
        warnings,
        errors
    );
    let field = if let Some(field) = fields
        .iter()
        .find(|TypedStructField { name, .. }| name.as_str() == field_to_access.as_str())
    {
        field
    } else {
        errors.push(CompileError::FieldNotFound {
            span: field_to_access.span().clone(),
            available_fields: fields
                .iter()
                .map(|TypedStructField { name, .. }| name.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
            field_name: field_to_access.clone(),
            struct_name: struct_name.to_string(),
        });
        return err(warnings, errors);
    };

    let exp = TypedExpression {
        expression: TypedExpressionVariant::StructFieldAccess {
            resolved_type_of_parent: parent.return_type,
            prefix: Box::new(parent),
            field_to_access: field.clone(),
        },
        return_type: field.r#type,
        is_constant: IsConstant::No,
        span,
    };
    ok(exp, warnings, errors)
}
