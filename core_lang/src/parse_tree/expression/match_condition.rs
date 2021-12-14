use super::Expression;

#[derive(Debug, Clone)]
pub(crate) enum MatchCondition<'sc> {
    CatchAll,
    Expression(Box<Expression<'sc>>),
}
