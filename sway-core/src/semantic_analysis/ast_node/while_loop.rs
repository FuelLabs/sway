use super::{TypedCodeBlock, TypedExpression};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TypedWhileLoop {
    pub(crate) condition: TypedExpression,
    pub(crate) body: TypedCodeBlock,
}

impl TypedWhileLoop {
    pub(crate) fn pretty_print(&self) -> String {
        format!("while loop on {}", self.condition.pretty_print())
    }
}
