use crate::language::parse_tree::Expression;

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub expr: Expression,
}
