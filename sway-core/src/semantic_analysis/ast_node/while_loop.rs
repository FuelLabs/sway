use std::fmt;

use crate::{error::ok, type_engine::ResolveTypes};

use super::{TypedCodeBlock, TypedExpression};

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
    fn resolve_types(
        &mut self,
        type_arguments: Vec<crate::TypeArgument>,
        enforce_type_arguments: super::EnforceTypeArguments,
        namespace: &mut crate::namespace::Root,
        module_path: &crate::namespace::Path,
    ) -> crate::CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        self.condition
            .resolve_types(vec![], enforce_type_arguments, namespace, module_path)
            .ok(&mut warnings, &mut errors);
        self.body
            .resolve_types(vec![], enforce_type_arguments, namespace, module_path)
            .ok(&mut warnings, &mut errors);
        ok((), warnings, errors)
    }
}
