use crate::{
    error::*,
    namespace::Items,
    parse_tree::*,
    semantic_analysis::{
        declaration::EnforceTypeArguments, insert_type_parameters, CopyTypes, TypeMapping,
    },
    type_engine::*,
    CompileError, CompileResult, Ident, Namespace,
};
use fuels_types::Property;
use std::hash::{Hash, Hasher};
use sway_types::Span;

use super::{monomorphize_inner, CreateTypeId, MonomorphizeHelper};

#[derive(Clone, Debug, Eq)]
pub struct TypedEnumDeclaration {
    pub name: Ident,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub variants: Vec<TypedEnumVariant>,
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
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
    }
}

impl CreateTypeId for TypedEnumDeclaration {
    fn create_type_id(&self) -> TypeId {
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
        monomorphize_inner(self, type_mapping, namespace)
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
        let mut namespace = namespace.clone();

        // insert type parameters as Unknown types
        let type_mapping = insert_type_parameters(&decl.type_parameters);

        // update the types in the type parameters
        let new_type_parameters = decl
            .type_parameters
            .into_iter()
            .map(|mut type_parameter| {
                type_parameter.type_id = match look_up_type_id(type_parameter.type_id)
                    .matches_type_parameter(&type_mapping)
                {
                    Some(matching_id) => {
                        insert_type(TypeInfo::Ref(matching_id, type_parameter.span()))
                    }
                    None => check!(
                        namespace.resolve_type_with_self(
                            look_up_type_id(type_parameter.type_id),
                            self_type,
                            &type_parameter.span(),
                            EnforceTypeArguments::Yes
                        ),
                        insert_type(TypeInfo::ErrorRecovery),
                        warnings,
                        errors,
                    ),
                };
                type_parameter
            })
            .collect::<Vec<_>>();

        // insert the generics into the decl namespace and
        // check to see if the type parameters shadow one another
        for type_parameter in new_type_parameters.iter() {
            check!(
                namespace.insert_symbol(type_parameter.name_ident.clone(), type_parameter.into()),
                continue,
                warnings,
                errors
            );
        }

        // type check the variants
        let mut variants_buf = vec![];
        for variant in decl.variants {
            variants_buf.push(check!(
                TypedEnumVariant::type_check(
                    variant.clone(),
                    &mut namespace,
                    self_type,
                    variant.span,
                    &type_mapping
                ),
                continue,
                warnings,
                errors
            ));
        }

        // create the enum decl
        let decl = TypedEnumDeclaration {
            name: decl.name.clone(),
            type_parameters: new_type_parameters,
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
    pub name: Ident,
    pub r#type: TypeId,
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
        self.r#type.update_type(type_mapping, &self.span);
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
            Some(matching_id) => insert_type(TypeInfo::Ref(matching_id, span)),
            None => {
                check!(
                    namespace.resolve_type_with_self(
                        variant.r#type.clone(),
                        self_type,
                        &span,
                        EnforceTypeArguments::Yes
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
