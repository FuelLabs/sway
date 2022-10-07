use crate::{language::ty::TyExpression, type_system::*};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TyReturnStatement {
    pub expr: TyExpression,
}

impl CopyTypes for TyReturnStatement {
    /// Makes a fresh copy of all types contained in this statement.
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.expr.copy_types(type_mapping);
    }
}
