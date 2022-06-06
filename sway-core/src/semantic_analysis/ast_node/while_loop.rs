use super::{TypedCodeBlock, TypedExpression};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedWhileLoop {
    pub condition: TypedExpression,
    pub body: TypedCodeBlock,
}

impl std::fmt::Display for TypedWhileLoop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "while loop on {}", self.condition)
    }
}
