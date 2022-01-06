use crate::semantic_analysis::TypedExpression;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) enum TypedMatchCondition {
    CatchAll,
    Expression(Box<TypedExpression>),
}
