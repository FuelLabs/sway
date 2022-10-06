use sway_types::{Ident, Span, Spanned};

use crate::{
    error::{err, ok},
    semantic_analysis::{IsConstant, TyExpression, TyExpressionVariant},
    type_system::look_up_type_id,
    CompileResult,
};

pub(crate) fn instantiate_struct_field_access(
    parent: TyExpression,
    field_to_access: Ident,
    span: Span,
) -> CompileResult<TyExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let field_instantiation_span = field_to_access.span();
    let field = check!(
        look_up_type_id(parent.return_type).apply_subfields(&[field_to_access], &parent.span),
        return err(warnings, errors),
        warnings,
        errors
    );
    let exp = TyExpression {
        expression: TyExpressionVariant::StructFieldAccess {
            resolved_type_of_parent: parent.return_type,
            prefix: Box::new(parent),
            field_to_access: field.clone(),
            field_instantiation_span,
        },
        return_type: field.type_id,
        is_constant: IsConstant::No,
        span,
    };
    ok(exp, warnings, errors)
}
