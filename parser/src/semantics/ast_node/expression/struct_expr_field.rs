use crate::semantics::TypedExpression;
use crate::Ident;

#[derive(Clone, Debug)]
pub(crate) struct TypedStructExpressionField<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) value: TypedExpression<'sc>,
}
