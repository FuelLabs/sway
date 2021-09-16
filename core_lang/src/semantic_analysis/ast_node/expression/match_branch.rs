use super::*;

#[derive(Clone, Debug)]
pub(crate) struct TypedMatchBranch<'sc> {
    pub(crate) pattern: TypedMatchPattern<'sc>,
    pub(crate) result: TypedExpression<'sc>,
}
