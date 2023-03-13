use sway_types::{Ident, Span, Spanned};

use crate::{
    error::{err, ok},
    language::ty,
    CompileResult, Engines,
};

pub(crate) fn instantiate_struct_field_access(
    engines: Engines<'_>,
    parent: ty::TyExpression,
    field_to_access: Ident,
    span: Span,
) -> CompileResult<ty::TyExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let type_engine = engines.te();
    let struct_type_id = type_engine.get(parent.return_type);
    let field_instantiation_span = field_to_access.span();
    let field = check!(
        struct_type_id.apply_subfields(engines, &[field_to_access], &parent.span),
        return err(warnings, errors),
        warnings,
        errors
    );
    let struct_ref = check!(
        struct_type_id.expect_struct(engines, &span),
        return err(warnings, errors),
        warnings,
        errors
    );
    let exp = ty::TyExpression {
        expression: ty::TyExpressionVariant::StructFieldAccess {
            struct_ref,
            prefix: Box::new(parent),
            field_to_access: field.clone(),
            field_instantiation_span,
        },
        return_type: field.type_argument.type_id,
        span,
    };
    ok(exp, warnings, errors)
}
