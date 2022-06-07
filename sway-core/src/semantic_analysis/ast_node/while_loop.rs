use std::fmt;

use super::{TypedCodeBlock, TypedExpression};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedWhileLoop {
    pub condition: TypedExpression,
    pub body: TypedCodeBlock,
}

impl fmt::Display for TypedWhileLoop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "while loop on {}", self.condition)
    }
}
