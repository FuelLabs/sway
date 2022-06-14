use crate::{
    namespace::{Path, Root},
    type_engine::{ResolveTypes, TypeId},
    CompileResult, TypeArgument,
};

use super::{CopyTypes, EnforceTypeArguments, TypeMapping, TypedExpression};

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
    fn resolve_type_with_self(
        &mut self,
        _type_arguments: Vec<TypeArgument>,
        enforce_type_arguments: EnforceTypeArguments,
        self_type: TypeId,
        namespace: &mut Root,
        module_path: &Path,
    ) -> CompileResult<()> {
        self.expr.resolve_type_with_self(
            vec![],
            enforce_type_arguments,
            self_type,
            namespace,
            module_path,
        )
    }

    fn resolve_type_without_self(
        &mut self,
        _type_arguments: Vec<TypeArgument>,
        namespace: &mut Root,
        module_path: &Path,
    ) -> CompileResult<()> {
        self.expr
            .resolve_type_without_self(vec![], namespace, module_path)
    }
}
