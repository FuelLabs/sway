use super::TypedExpression;

#[derive(Clone, Debug)]
pub(crate) struct TypedReturnStatement<'sc> {
    pub(crate) expr: TypedExpression<'sc>,
}
