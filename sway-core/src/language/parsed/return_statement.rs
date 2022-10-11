use crate::language::parsed::Expression;

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub expr: Expression,
}
