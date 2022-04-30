use sway_types::Span;

use crate::{
    error::{err, ok},
    semantic_analysis::{IsConstant, TypedExpression, TypedExpressionVariant},
    CompileError, CompileResult, NamespaceRef, NamespaceWrapper,
};

pub(crate) fn instantiate_tuple_index_access(
    parent: TypedExpression,
    index: usize,
    index_span: Span,
    span: Span,
    namespace: NamespaceRef,
) -> CompileResult<TypedExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut tuple_elem_to_access = None;
    let tuple_elems = check!(
        namespace.get_tuple_elems(parent.return_type, parent.span.as_str(), &parent.span),
        return err(warnings, errors),
        warnings,
        errors
    );
    for (pos, elem) in tuple_elems.iter().enumerate() {
        if pos == index {
            tuple_elem_to_access = Some(elem.clone());
        }
    }
    let tuple_elem_to_access = match tuple_elem_to_access {
        Some(tuple_elem_to_access) => tuple_elem_to_access,
        None => {
            errors.push(CompileError::TupleOutOfBounds {
                index,
                count: tuple_elems.len(),
                span: index_span,
            });
            return err(warnings, errors);
        }
    };
    let exp = TypedExpression {
        expression: TypedExpressionVariant::TupleIndexAccess {
            resolved_type_of_parent: parent.return_type,
            prefix: Box::new(parent),
            elem_to_access_num: index,
            elem_to_access_span: index_span,
        },
        return_type: tuple_elem_to_access.type_id,
        is_constant: IsConstant::No,
        span,
    };
    ok(exp, warnings, errors)
}
