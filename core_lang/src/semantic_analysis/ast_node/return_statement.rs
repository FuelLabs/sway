use super::TypedExpression;

#[derive(Clone, Debug)]
pub struct TypedReturnStatement<'sc> {
    pub(crate) expr: TypedExpression<'sc>,
}
