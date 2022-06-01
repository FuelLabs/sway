use super::{TypedCodeBlock, TypedExpression};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedWhileLoop {
    pub condition: TypedExpression,
    pub body: TypedCodeBlock,
}

impl TypedWhileLoop {
    pub(crate) fn pretty_print(&self) -> String {
        format!("while loop on {}", self.condition.pretty_print())
    }
}
