use crate::Ident;
use crate::{semantic_analysis::*, type_engine::*};

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
    fn resolve_types(
        &mut self,
        _type_arguments: Vec<crate::TypeArgument>,
        enforce_type_arguments: EnforceTypeArguments,
        namespace: &mut namespace::Root,
        module_path: &namespace::Path,
    ) -> crate::CompileResult<()> {
        self.value
            .resolve_types(vec![], enforce_type_arguments, namespace, module_path)
    }
}
