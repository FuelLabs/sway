use crate::{
    error::{err, ok},
    namespace::Items,
    parse_tree::*,
    semantic_analysis::{
        ast_node::copy_types::TypeMapping, declaration::EnforceTypeArguments,
        insert_type_parameters, CopyTypes,
    },
    type_engine::*,
    CompileError, CompileResult, Ident, Namespace,
};
use fuels_types::Property;
use std::hash::{Hash, Hasher};
use sway_types::Span;

use super::{CreateTypeId, MonomorphizeHelper};

#[derive(Clone, Debug, Eq)]
pub struct TypedEnumDeclaration {
    pub(crate) name: Ident,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub(crate) variants: Vec<TypedEnumVariant>,
    pub(crate) span: Span,
    pub(crate) visibility: Visibility,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypedEnumDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.type_parameters == other.type_parameters
            && self.variants == other.variants
            && self.visibility == other.visibility
    }
}

impl CopyTypes for TypedEnumDeclaration {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.variants
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
    }
}

impl CreateTypeId for TypedEnumDeclaration {
    fn type_id(&self) -> TypeId {
        insert_type(TypeInfo::Enum {
            name: self.name.clone(),
            variant_types: self.variants.clone(),
            type_parameters: self.type_parameters.clone(),
        })
    }
}

impl MonomorphizeHelper for TypedEnumDeclaration {
    type Output = TypedEnumDeclaration;

    fn type_parameters(&self) -> &[TypeParameter] {
        &self.type_parameters
    }

    fn name(&self) -> &Ident {
        &self.name
    }

    fn span(&self) -> &Span {
        &self.span
    }

    fn monomorphize_inner(self, type_mapping: &TypeMapping, namespace: &mut Items) -> Self::Output {
        let old_type_id = self.type_id();
        let mut new_decl = self;
        new_decl.copy_types(type_mapping);
        namespace.copy_methods_to_type(
            look_up_type_id(old_type_id),
            look_up_type_id(new_decl.type_id()),
            type_mapping,
        );
        new_decl
    }
}

impl TypedEnumDeclaration {
    pub fn type_check(
        decl: EnumDeclaration,
        namespace: &mut Namespace,
        self_type: TypeId,
    ) -> CompileResult<TypedEnumDeclaration> {
        let mut errors = vec![];
        let mut warnings = vec![];

        // create a namespace for the decl, used to create a scope for generics
        let mut decl_namespace = namespace.clone();

        // insert the generics into the decl namespace and
        // check to see if the type parameters shadow one another
        for type_parameter in decl.type_parameters.iter() {
            check!(
                decl_namespace
                    .insert_symbol(type_parameter.name_ident.clone(), type_parameter.into()),
                continue,
                warnings,
                errors
            );
        }

        let mut variants_buf = vec![];
        let type_mapping = insert_type_parameters(&decl.type_parameters);
        for variant in decl.variants {
            variants_buf.push(check!(
                TypedEnumVariant::type_check(
                    variant.clone(),
                    &mut decl_namespace,
                    self_type,
                    variant.span,
                    &type_mapping
                ),
                continue,
                warnings,
                errors
            ));
        }

        let decl = TypedEnumDeclaration {
            name: decl.name.clone(),
            type_parameters: decl.type_parameters.clone(),
            variants: variants_buf,
            span: decl.span.clone(),
            visibility: decl.visibility,
        };
        ok(decl, warnings, errors)
    }

    pub(crate) fn expect_variant_from_name(
        &self,
        variant_name: &Ident,
    ) -> CompileResult<&TypedEnumVariant> {
        let warnings = vec![];
        let mut errors = vec![];
        match self
            .variants
            .iter()
            .find(|x| x.name.as_str() == variant_name.as_str())
        {
            Some(variant) => ok(variant, warnings, errors),
            None => {
                errors.push(CompileError::UnknownEnumVariant {
                    enum_name: self.name.clone(),
                    variant_name: variant_name.clone(),
                    span: self.span.clone(),
                });
                err(warnings, errors)
            }
        }
    }
}

#[derive(Debug, Clone, Eq)]
pub struct TypedEnumVariant {
    pub(crate) name: Ident,
    pub(crate) r#type: TypeId,
    pub(crate) tag: usize,
    pub(crate) span: Span,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl Hash for TypedEnumVariant {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        look_up_type_id(self.r#type).hash(state);
        self.tag.hash(state);
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypedEnumVariant {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && look_up_type_id(self.r#type) == look_up_type_id(other.r#type)
            && self.tag == other.tag
    }
}

impl CopyTypes for TypedEnumVariant {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.r#type = if let Some(matching_id) =
            look_up_type_id(self.r#type).matches_type_parameter(type_mapping)
        {
            insert_type(TypeInfo::Ref(matching_id))
        } else {
            insert_type(look_up_type_id_raw(self.r#type))
        };
    }
}

impl TypedEnumVariant {
    pub(crate) fn type_check(
        variant: EnumVariant,
        namespace: &mut Namespace,
        self_type: TypeId,
        span: Span,
        type_mapping: &TypeMapping,
    ) -> CompileResult<TypedEnumVariant> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let enum_variant_type = match variant.r#type.matches_type_parameter(type_mapping) {
            Some(matching_id) => insert_type(TypeInfo::Ref(matching_id)),
            None => {
                check!(
                    namespace.resolve_type_with_self(
                        variant.r#type.clone(),
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
                name: variant.name.clone(),
                r#type: enum_variant_type,
                tag: variant.tag,
                span: variant.span,
            },
            vec![],
            errors,
        )
    }

    pub fn generate_json_abi(&self) -> Property {
        Property {
            name: self.name.to_string(),
            type_field: self.r#type.json_abi_str(),
            components: self.r#type.generate_json_abi(),
        }
    }
}
