use super::{CopyTypes, TypeMapping};

use crate::language::ty;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TyReturnStatement {
    pub expr: ty::TyExpression,
}

impl CopyTypes for TyReturnStatement {
    /// Makes a fresh copy of all types contained in this statement.
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.expr.copy_types(type_mapping);
    }
}
