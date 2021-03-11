use super::{TypedCodeBlock, TypedExpression};
#[derive(Clone, Debug)]
pub(crate) struct TypedWhileLoop<'sc> {
    pub(crate) condition: TypedExpression<'sc>,
    pub(crate) body: TypedCodeBlock<'sc>,
}
