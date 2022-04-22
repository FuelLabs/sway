use super::TypedExpression;
use crate::{type_engine::TypeId, TypeParameter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TypedReturnStatement {
    pub(crate) expr: TypedExpression,
}

impl TypedReturnStatement {
    /// Makes a fresh copy of all types contained in this statement.
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.expr.copy_types(type_mapping);
    }
}
