use super::*;
use crate::semantics::ast_node::TypedCodeBlock;
use either::Either;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct TypedMatchBranch<'sc> {
    condition: TypedMatchCondition<'sc>,
    result: Either<TypedCodeBlock<'sc>, TypedExpression<'sc>>,
}
