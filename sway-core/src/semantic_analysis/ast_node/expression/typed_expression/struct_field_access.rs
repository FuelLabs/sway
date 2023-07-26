use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{Ident, Span, Spanned};

use crate::{language::ty, Engines};

pub(crate) fn instantiate_struct_field_access(
    handler: &Handler,
    engines: &Engines,
    parent: ty::TyExpression,
    field_to_access: Ident,
    span: Span,
) -> Result<ty::TyExpression, ErrorEmitted> {
    let type_engine = engines.te();
    let field_instantiation_span = field_to_access.span();
    let field = type_engine.get(parent.return_type).apply_subfields(
        handler,
        engines,
        &[field_to_access],
        &parent.span,
    )?;
    let exp = ty::TyExpression {
        expression: ty::TyExpressionVariant::StructFieldAccess {
            resolved_type_of_parent: parent.return_type,
            prefix: Box::new(parent),
            field_to_access: field.clone(),
            field_instantiation_span,
        },
        return_type: field.type_argument.type_id,
        span,
    };
    Ok(exp)
}
