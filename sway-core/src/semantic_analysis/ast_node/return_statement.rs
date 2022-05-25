use super::{CopyTypes, TypeMapping, TypedExpression};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedReturnStatement {
    pub expr: TypedExpression,
}

impl CopyTypes for TypedReturnStatement {
    /// Makes a fresh copy of all types contained in this statement.
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.expr.copy_types(type_mapping);
    }
}
