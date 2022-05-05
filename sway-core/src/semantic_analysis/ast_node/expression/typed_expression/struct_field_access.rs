use sway_types::{Ident, Span};

use crate::{
    error::{err, ok},
    semantic_analysis::{IsConstant, TypedExpression, TypedExpressionVariant, TypedStructField},
    CompileResult, NamespaceRef, NamespaceWrapper,
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
        namespace.expect_struct_fields_from_type_id(
            parent.return_type,
            parent.span.as_str(),
            &parent.span
        ),
        return err(warnings, errors),
        warnings,
        errors
    );
    let field = check!(
        TypedStructField::expect_field_from_fields(&struct_name, &fields, &field_to_access),
        return err(warnings, errors),
        warnings,
        errors
    );

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
