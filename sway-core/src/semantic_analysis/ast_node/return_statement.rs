use crate::type_engine::ResolveTypes;

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

impl ResolveTypes for TypedReturnStatement {
    fn resolve_types(
        &mut self,
        type_arguments: Vec<crate::TypeArgument>,
        enforce_type_arguments: super::EnforceTypeArguments,
        namespace: &mut crate::namespace::Root,
        module_path: &crate::namespace::Path,
    ) -> crate::CompileResult<()> {
        self.expr
            .resolve_types(vec![], enforce_type_arguments, namespace, module_path)
    }
}
