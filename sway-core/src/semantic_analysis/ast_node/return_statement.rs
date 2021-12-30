use super::TypedExpression;
use crate::{type_engine::TypeId, TypeParameter};

#[derive(Clone, Debug)]
pub(crate) struct TypedReturnStatement<'sc> {
    pub(crate) expr: TypedExpression<'sc>,
}

impl TypedReturnStatement<'_> {
    /// Makes a fresh copy of all types contained in this statement.
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.expr.copy_types(type_mapping);
    }
}
