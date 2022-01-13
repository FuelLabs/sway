use super::Expression;

#[derive(Debug, Clone)]
pub(crate) enum MatchCondition {
    CatchAll,
    Expression(Box<Expression>),
}
