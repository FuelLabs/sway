use crate::{
    CodeBlock, Expression,
};



/// A parsed while loop. Contains the `condition`, which is defined from an [Expression], and the `body` from a [CodeBlock].
#[derive(Debug, Clone)]
pub struct WhileLoop {
    pub condition: Expression,
    pub body: CodeBlock,
}
