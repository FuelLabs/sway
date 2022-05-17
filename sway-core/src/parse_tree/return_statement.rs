use crate::Expression;

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub expr: Expression,
}
