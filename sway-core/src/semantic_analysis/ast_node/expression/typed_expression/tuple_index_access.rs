use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::Span;

use crate::{language::ty, CompileError, Engines, TypeInfo};

pub(crate) fn instantiate_tuple_index_access(
    handler: &Handler,
    engines: &Engines,
    parent: ty::TyExpression,
    index: usize,
    index_span: Span,
    span: Span,
) -> Result<ty::TyExpression, ErrorEmitted> {
    let type_engine = engines.te();

    let mut current_prefix_te = Box::new(parent);
    let mut current_type = type_engine.get_unaliased(current_prefix_te.return_type);

    let prefix_type_id = current_prefix_te.return_type;
    let prefix_span = current_prefix_te.span.clone();

    // Create the prefix part of the final tuple element access expression.
    // This might be an expression that directly evaluates to a tuple type,
    // or an arbitrary number of dereferencing expressions where the last one
    // dereference to a tuple type.
    //
    // We will either hit a tuple at the end or return an error, so the
    // loop cannot be endless.
    while !current_type.is_tuple() {
        match &*current_type {
            TypeInfo::Ref {
                referenced_type, ..
            } => {
                let referenced_type_id = referenced_type.type_id();

                current_prefix_te = Box::new(ty::TyExpression {
                    expression: ty::TyExpressionVariant::Deref(current_prefix_te),
                    return_type: referenced_type_id,
                    span: prefix_span.clone(),
                });

                current_type = type_engine.get_unaliased(referenced_type_id);
            }
            TypeInfo::ErrorRecovery(err) => return Err(*err),
            _ => {
                return Err(
                    handler.emit_err(CompileError::TupleElementAccessOnNonTuple {
                        actually: engines.help_out(prefix_type_id).to_string(),
                        span: prefix_span,
                        index,
                        index_span,
                    }),
                )
            }
        };
    }

    let TypeInfo::Tuple(type_args) = &*current_type else {
        panic!("The current type must be a tuple.");
    };

    if type_args.len() <= index {
        return Err(handler.emit_err(CompileError::TupleIndexOutOfBounds {
            index,
            count: type_args.len(),
            tuple_type: engines.help_out(prefix_type_id).to_string(),
            span: index_span,
            prefix_span,
        }));
    }

    Ok(ty::TyExpression {
        expression: ty::TyExpressionVariant::TupleElemAccess {
            resolved_type_of_parent: current_prefix_te.return_type,
            prefix: current_prefix_te,
            elem_to_access_num: index,
            elem_to_access_span: index_span,
        },
        return_type: type_args[index].type_id(),
        span,
    })
}
