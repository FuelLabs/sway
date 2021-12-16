use crate::semantic_analysis::TypedExpression;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) enum TypedMatchCondition<'sc> {
    CatchAll,
    Expression(Box<TypedExpression<'sc>>),
}
