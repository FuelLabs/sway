use super::*;
use crate::semantic_analysis::ast_node::TypedCodeBlock;
use either::Either;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct TypedMatchBranch {
    condition: TypedMatchCondition,
    result: Either<TypedCodeBlock, TypedExpression>,
}
