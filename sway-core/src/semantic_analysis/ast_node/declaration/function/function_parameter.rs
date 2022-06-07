use crate::{
    error::ok, semantic_analysis::EnforceTypeArguments, type_engine::*, CompileResult,
    FunctionParameter, Ident, Namespace,
};

use sway_types::span::Span;

#[derive(Debug, Clone, Eq)]
pub struct TypedFunctionParameter {
    pub name: Ident,
    pub r#type: TypeId,
    pub(crate) type_span: Span,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypedFunctionParameter {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && look_up_type_id(self.r#type) == look_up_type_id(other.r#type)
    }
}

impl CopyTypes for TypedFunctionParameter {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.r#type.update_type(type_mapping, &self.type_span);
    }
}

impl TypedFunctionParameter {
    pub(crate) fn type_check(
        parameter: FunctionParameter,
        namespace: &mut Namespace,
        self_type: TypeId,
        enforce_type_arguments: EnforceTypeArguments,
    ) -> CompileResult<TypedFunctionParameter> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let type_id = check!(
            namespace.resolve_type_with_self(
                look_up_type_id(parameter.type_id),
                self_type,
                &parameter.type_span,
                enforce_type_arguments
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );
        let parameter = TypedFunctionParameter {
            name: parameter.name.clone(),
            r#type: type_id,
            type_span: parameter.type_span.clone(),
        };
        ok(parameter, warnings, errors)
    }
}
