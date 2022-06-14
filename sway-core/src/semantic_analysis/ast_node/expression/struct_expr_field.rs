use std::fmt;

use crate::namespace::{Path, Root};
use crate::{semantic_analysis::*, type_engine::*, TypeArgument};
use crate::{CompileResult, Ident};

#[derive(Clone, Debug, PartialEq)]
pub struct TypedStructExpressionField {
    pub name: Ident,
    pub value: TypedExpression,
}

impl CopyTypes for TypedStructExpressionField {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.value.copy_types(type_mapping);
    }
}

impl ResolveTypes for TypedStructExpressionField {
    fn resolve_type_with_self(
        &mut self,
        _type_arguments: Vec<TypeArgument>,
        enforce_type_arguments: EnforceTypeArguments,
        self_type: TypeId,
        namespace: &mut Root,
        module_path: &Path,
    ) -> CompileResult<()> {
        self.value.resolve_type_with_self(
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
        self.value
            .resolve_type_without_self(vec![], namespace, module_path)
    }
}

impl fmt::Display for TypedStructExpressionField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}
