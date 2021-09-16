use super::*;

#[derive(Clone, Debug)]
pub enum PatternVariant<'sc> {
    CatchAll,
    Expression(TypedExpression<'sc>),
}
