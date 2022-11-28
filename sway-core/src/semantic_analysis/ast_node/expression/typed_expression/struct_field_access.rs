use sway_types::{Ident, Span, Spanned};

use crate::{
    error::{err, ok},
    language::ty,
    CompileResult, TypeEngine,
};

pub(crate) fn instantiate_struct_field_access(
    type_engine: &TypeEngine,
    parent: ty::TyExpression,
    field_to_access: Ident,
    span: Span,
) -> CompileResult<ty::TyExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let field_instantiation_span = field_to_access.span();
    let field = check!(
        type_engine
            .look_up_type_id(parent.return_type)
            .apply_subfields(type_engine, &[field_to_access], &parent.span),
        return err(warnings, errors),
        warnings,
        errors
    );
    let exp = ty::TyExpression {
        expression: ty::TyExpressionVariant::StructFieldAccess {
            resolved_type_of_parent: parent.return_type,
            prefix: Box::new(parent),
            field_to_access: field.clone(),
            field_instantiation_span,
        },
        return_type: field.type_id,
        span,
    };
    ok(exp, warnings, errors)
}
