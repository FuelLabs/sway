use crate::semantics::TypedExpression;

#[derive(Clone, Debug)]
pub(crate) struct TypedStructExpressionField<'sc> {
    pub(crate) name: &'sc str,
    pub(crate) value: TypedExpression<'sc>,
}
