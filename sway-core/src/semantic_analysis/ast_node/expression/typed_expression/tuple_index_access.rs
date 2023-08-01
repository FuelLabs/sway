use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::Span;

use crate::{language::ty, CompileError, Engines};

pub(crate) fn instantiate_tuple_index_access(
    handler: &Handler,
    engines: &Engines,
    parent: ty::TyExpression,
    index: usize,
    index_span: Span,
    span: Span,
) -> Result<ty::TyExpression, ErrorEmitted> {
    let type_engine = engines.te();
    let mut tuple_type_arg_to_access = None;
    let type_info = type_engine.get(parent.return_type);
    let type_args = type_info.expect_tuple(handler, engines, parent.span.as_str(), &parent.span)?;
    for (pos, type_arg) in type_args.iter().enumerate() {
        if pos == index {
            tuple_type_arg_to_access = Some(type_arg.clone());
        }
    }
    let tuple_type_arg_to_access = match tuple_type_arg_to_access {
        Some(tuple_type_arg_to_access) => tuple_type_arg_to_access,
        None => {
            return Err(handler.emit_err(CompileError::TupleIndexOutOfBounds {
                index,
                count: type_args.len(),
                span: index_span,
            }));
        }
    };
    let exp = ty::TyExpression {
        expression: ty::TyExpressionVariant::TupleElemAccess {
            resolved_type_of_parent: parent.return_type,
            prefix: Box::new(parent),
            elem_to_access_num: index,
            elem_to_access_span: index_span,
        },
        return_type: tuple_type_arg_to_access.type_id,
        span,
    };
    Ok(exp)
}
