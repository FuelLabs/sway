use super::{TypedCodeBlock, TypedExpression};

#[derive(Clone, Debug)]
pub(crate) struct TypedWhileLoop<'sc> {
    pub(crate) condition: TypedExpression<'sc>,
    pub(crate) body: TypedCodeBlock<'sc>,
}

impl<'sc> TypedWhileLoop<'sc> {
    pub(crate) fn pretty_print(&self) -> String {
        format!("while loop on {}", self.condition.pretty_print())
    }
}
