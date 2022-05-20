use crate::{
    error::*,
    parse_tree::{declaration::TypeParameter, Visibility},
    semantic_analysis::{
        ast_node::{TypedEnumDeclaration, TypedEnumVariant},
        insert_type_parameters,
        namespace::Namespace,
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

impl EnumDeclaration {
    /// Looks up the various TypeInfos in the [Namespace] to see if they are generic or refer to
    /// something.
    pub(crate) fn to_typed_decl(
        &self,
        namespace: &mut Namespace,
        self_type: TypeId,
    ) -> TypedEnumDeclaration {
        let mut errors = vec![];
        let mut warnings = vec![];

        let mut variants_buf = vec![];
        let type_mapping = insert_type_parameters(&self.type_parameters);
        for variant in &self.variants {
            variants_buf.push(check!(
                variant.to_typed_decl(namespace, self_type, variant.span.clone(), &type_mapping),
                continue,
                warnings,
                errors
            ));
        }
        TypedEnumDeclaration {
            name: self.name.clone(),
            type_parameters: self.type_parameters.clone(),
            variants: variants_buf,
            span: self.span.clone(),
            visibility: self.visibility,
        }
    }
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
        let enum_variant_type =
            if let Some(matching_id) = self.r#type.matches_type_parameter(type_mapping) {
                insert_type(TypeInfo::Ref(matching_id))
            } else {
                check!(
                    namespace.resolve_type_with_self(self.r#type.clone(), self_type, span, false),
                    insert_type(TypeInfo::ErrorRecovery),
                    warnings,
                    errors,
                )
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
