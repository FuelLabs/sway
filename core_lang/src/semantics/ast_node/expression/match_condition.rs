use crate::semantics::TypedExpression;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) enum TypedMatchCondition<'sc> {
    CatchAll,
    Expression(TypedExpression<'sc>),
}
