use super::TypedExpression;

#[derive(Clone, Debug)]
pub(crate) struct TypedReturnStatement<'sc> {
    pub(crate) expr: TypedExpression<'sc>,
}

impl TypedReturnStatement<'_> {
    /// Makes a fresh copy of all types contained in this statement.
    pub(crate) fn copy_types(&mut self) {
        self.expr.copy_types();
    }
}
