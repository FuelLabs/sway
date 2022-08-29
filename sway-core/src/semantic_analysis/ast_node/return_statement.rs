use crate::types::{CompileWrapper, ToCompileWrapper};

use super::{CopyTypes, TypeMapping, TypedExpression};

#[derive(Clone, Debug)]
pub struct TypedReturnStatement {
    pub expr: TypedExpression,
}

impl PartialEq for CompileWrapper<'_, TypedReturnStatement> {
    fn eq(&self, other: &Self) -> bool {
        let CompileWrapper {
            inner: me,
            declaration_engine: de,
        } = self;
        let CompileWrapper {
            inner: them,
            declaration_engine: _,
        } = other;
        me.expr.wrap(de) == them.expr.wrap(de)
    }
}

impl CopyTypes for TypedReturnStatement {
    /// Makes a fresh copy of all types contained in this statement.
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.expr.copy_types(type_mapping);
    }
}
