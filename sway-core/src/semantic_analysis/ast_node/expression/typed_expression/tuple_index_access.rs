use sway_types::Span;

use crate::{
    error::{err, ok},
    language::ty,
    CompileError, CompileResult, TypeEngine,
};

pub(crate) fn instantiate_tuple_index_access(
    type_engine: &TypeEngine,
    parent: ty::TyExpression,
    index: usize,
    index_span: Span,
    span: Span,
) -> CompileResult<ty::TyExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut tuple_type_arg_to_access = None;
    let type_info = type_engine.look_up_type_id(parent.return_type);
    let type_args = check!(
        type_info.expect_tuple(type_engine, parent.span.as_str(), &parent.span),
        return err(warnings, errors),
        warnings,
        errors
    );
    for (pos, type_arg) in type_args.iter().enumerate() {
        if pos == index {
            tuple_type_arg_to_access = Some(type_arg.clone());
        }
    }
    let tuple_type_arg_to_access = match tuple_type_arg_to_access {
        Some(tuple_type_arg_to_access) => tuple_type_arg_to_access,
        None => {
            errors.push(CompileError::TupleIndexOutOfBounds {
                index,
                count: type_args.len(),
                span: index_span,
            });
            return err(warnings, errors);
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
    ok(exp, warnings, errors)
}
