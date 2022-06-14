use std::fmt;

use crate::{
    error::ok,
    namespace::{Path, Root},
    type_engine::{ResolveTypes, TypeId},
    CompileResult, TypeArgument,
};

use super::{EnforceTypeArguments, TypedCodeBlock, TypedExpression};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedWhileLoop {
    pub condition: TypedExpression,
    pub body: TypedCodeBlock,
}

impl fmt::Display for TypedWhileLoop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "while loop on {}", self.condition)
    }
}

impl ResolveTypes for TypedWhileLoop {
    fn resolve_type_with_self(
        &mut self,
        _type_arguments: Vec<TypeArgument>,
        enforce_type_arguments: EnforceTypeArguments,
        self_type: TypeId,
        namespace: &mut Root,
        module_path: &Path,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        self.condition
            .resolve_type_with_self(
                vec![],
                enforce_type_arguments,
                self_type,
                namespace,
                module_path,
            )
            .ok(&mut warnings, &mut errors);
        self.body
            .resolve_type_with_self(
                vec![],
                enforce_type_arguments,
                self_type,
                namespace,
                module_path,
            )
            .ok(&mut warnings, &mut errors);
        ok((), warnings, errors)
    }

    fn resolve_type_without_self(
        &mut self,
        _type_arguments: Vec<TypeArgument>,
        namespace: &mut Root,
        module_path: &Path,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        self.condition
            .resolve_type_without_self(vec![], namespace, module_path)
            .ok(&mut warnings, &mut errors);
        self.body
            .resolve_type_without_self(vec![], namespace, module_path)
            .ok(&mut warnings, &mut errors);
        ok((), warnings, errors)
    }
}
