use crate::{error::*, parse_tree::*, semantic_analysis::namespace, type_engine::*, Ident};
use fuels_types::Property;
use std::hash::{Hash, Hasher};
use sway_types::Span;

use super::insert_type_parameters;

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

impl TypedEnumDeclaration {
    pub(crate) fn monomorphize(&self, namespace: &mut namespace::Items) -> Self {
        let type_mapping = insert_type_parameters(&self.type_parameters);
        Self::monomorphize_inner(self, namespace, &type_mapping)
    }

    pub(crate) fn monomorphize_with_type_arguments(
        &self,
        namespace: &mut namespace::Items,
        type_arguments: &[TypeArgument],
        self_type: Option<TypeId>,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let type_mapping = insert_type_parameters(&self.type_parameters);
        let mut new_decl = Self::monomorphize_inner(self, namespace, &type_mapping);
        let type_arguments_span = type_arguments
            .iter()
            .map(|x| x.span.clone())
            .reduce(Span::join)
            .unwrap_or_else(|| self.span.clone());
        if type_mapping.len() != type_arguments.len() {
            errors.push(CompileError::IncorrectNumberOfTypeArguments {
                given: type_arguments.len(),
                expected: type_mapping.len(),
                span: type_arguments_span,
            });
            return err(warnings, errors);
        }
        for ((_, interim_type), type_argument) in type_mapping.iter().zip(type_arguments.iter()) {
            match self_type {
                Some(self_type) => {
                    let (mut new_warnings, new_errors) = unify_with_self(
                        *interim_type,
                        type_argument.type_id,
                        self_type,
                        &type_argument.span,
                        "Type argument is not assignable to generic type parameter.",
                    );
                    warnings.append(&mut new_warnings);
                    errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
                }
                None => {
                    let (mut new_warnings, new_errors) = unify(
                        *interim_type,
                        type_argument.type_id,
                        &type_argument.span,
                        "Type argument is not assignable to generic type parameter.",
                    );
                    warnings.append(&mut new_warnings);
                    errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
                }
            }
        }
        // associate the type arguments with the parameters in the struct decl
        new_decl
            .type_parameters
            .iter_mut()
            .zip(type_arguments.iter())
            .for_each(
                |(
                    TypeParameter {
                        ref mut type_id, ..
                    },
                    arg,
                )| {
                    *type_id = arg.type_id;
                },
            );
        // perform the monomorphization
        ok(new_decl, warnings, errors)
    }

    fn monomorphize_inner(
        &self,
        namespace: &mut namespace::Items,
        type_mapping: &[(TypeParameter, TypeId)],
    ) -> Self {
        let old_type_id = self.type_id();
        let mut new_decl = self.clone();
        new_decl.copy_types(type_mapping);
        namespace.copy_methods_to_type(
            look_up_type_id(old_type_id),
            look_up_type_id(new_decl.type_id()),
            type_mapping,
        );
        new_decl
    }

    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.variants
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
    }

    pub(crate) fn type_id(&self) -> TypeId {
        insert_type(TypeInfo::Enum {
            name: self.name.clone(),
            variant_types: self.variants.clone(),
            type_parameters: self.type_parameters.clone(),
        })
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

impl TypedEnumVariant {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.r#type = if let Some(matching_id) =
            look_up_type_id(self.r#type).matches_type_parameter(type_mapping)
        {
            insert_type(TypeInfo::Ref(matching_id))
        } else {
            insert_type(look_up_type_id_raw(self.r#type))
        };
    }

    pub fn generate_json_abi(&self) -> Property {
        Property {
            name: self.name.to_string(),
            type_field: self.r#type.json_abi_str(),
            components: self.r#type.generate_json_abi(),
        }
    }
}
