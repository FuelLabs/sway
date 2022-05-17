use crate::{
    error::*,
    parse_tree::{declaration::TypeParameter, Visibility},
    semantic_analysis::{
        ast_node::TypedEnumVariant, declaration::EnforceTypeArguments, namespace::Namespace,
    },
    type_engine::*,
};

use sway_types::{ident::Ident, span::Span};

#[derive(Debug, Clone)]
pub struct EnumDeclaration {
    pub name: Ident,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub variants: Vec<EnumVariant>,
    pub(crate) span: Span,
    pub visibility: Visibility,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: Ident,
    pub(crate) r#type: TypeInfo,
    pub(crate) tag: usize,
    pub(crate) span: Span,
}

impl EnumVariant {
    pub(crate) fn to_typed_decl(
        &self,
        namespace: &mut Namespace,
        self_type: TypeId,
        span: Span,
        type_mapping: &[(TypeParameter, TypeId)],
    ) -> CompileResult<TypedEnumVariant> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let enum_variant_type = match self.r#type.matches_type_parameter(type_mapping) {
            Some(matching_id) => insert_type(TypeInfo::Ref(matching_id)),
            None => {
                check!(
                    namespace.resolve_type_with_self(
                        self.r#type.clone(),
                        self_type,
                        &span,
                        EnforceTypeArguments::No
                    ),
                    insert_type(TypeInfo::ErrorRecovery),
                    warnings,
                    errors,
                )
            }
        };
        ok(
            TypedEnumVariant {
                name: self.name.clone(),
                r#type: enum_variant_type,
                tag: self.tag,
                span: self.span.clone(),
            },
            vec![],
            errors,
        )
    }
}
