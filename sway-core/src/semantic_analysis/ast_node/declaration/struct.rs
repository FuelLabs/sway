use crate::{
    error::{err, ok},
    namespace::Items,
    parse_tree::*,
    semantic_analysis::{ast_node::copy_types::TypeMapping, insert_type_parameters, CopyTypes},
    type_engine::*,
    CompileError, CompileResult, Ident, Namespace,
};
use fuels_types::Property;
use std::hash::{Hash, Hasher};
use sway_types::Span;

use super::{monomorphize_inner, CreateTypeId, EnforceTypeArguments, MonomorphizeHelper};

#[derive(Clone, Debug, Eq)]
pub struct TypedStructDeclaration {
    pub(crate) name: Ident,
    pub(crate) fields: Vec<TypedStructField>,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub(crate) visibility: Visibility,
    pub(crate) span: Span,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypedStructDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.fields == other.fields
            && self.type_parameters == other.type_parameters
            && self.visibility == other.visibility
    }
}

impl CopyTypes for TypedStructDeclaration {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.fields
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
    }
}

impl CreateTypeId for TypedStructDeclaration {
    fn create_type_id(&self) -> TypeId {
        insert_type(TypeInfo::Struct {
            name: self.name.clone(),
            fields: self.fields.clone(),
            type_parameters: self.type_parameters.clone(),
        })
    }
}

impl MonomorphizeHelper for TypedStructDeclaration {
    type Output = TypedStructDeclaration;

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

impl TypedStructDeclaration {
    pub(crate) fn type_check(
        decl: StructDeclaration,
        namespace: &mut Namespace,
        self_type: TypeId,
    ) -> CompileResult<TypedStructDeclaration> {
        let mut warnings = vec![];
        let mut errors = vec![];

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

        // create the type parameters type mapping of custom types to generic types
        let type_mapping = insert_type_parameters(&decl.type_parameters);
        let fields = decl
            .fields
            .into_iter()
            .map(|field| {
                let StructField {
                    name,
                    r#type,
                    span,
                    type_span,
                } = field;
                let r#type = match r#type.matches_type_parameter(&type_mapping) {
                    Some(matching_id) => insert_type(TypeInfo::Ref(matching_id)),
                    None => check!(
                        decl_namespace.resolve_type_with_self(
                            r#type,
                            self_type,
                            &type_span,
                            EnforceTypeArguments::No
                        ),
                        insert_type(TypeInfo::ErrorRecovery),
                        warnings,
                        errors,
                    ),
                };
                TypedStructField { name, r#type, span }
            })
            .collect::<Vec<_>>();

        // create the struct decl
        let decl = TypedStructDeclaration {
            name: decl.name.clone(),
            type_parameters: decl.type_parameters.clone(),
            fields,
            visibility: decl.visibility,
            span: decl.span,
        };

        ok(decl, warnings, errors)
    }

    pub(crate) fn expect_field(&self, field_to_access: &Ident) -> CompileResult<&TypedStructField> {
        let warnings = vec![];
        let mut errors = vec![];
        match self
            .fields
            .iter()
            .find(|TypedStructField { name, .. }| name.as_str() == field_to_access.as_str())
        {
            Some(field) => ok(field, warnings, errors),
            None => {
                errors.push(CompileError::FieldNotFound {
                    available_fields: self
                        .fields
                        .iter()
                        .map(|TypedStructField { name, .. }| name.to_string())
                        .collect::<Vec<_>>()
                        .join("\n"),
                    field_name: field_to_access.clone(),
                    struct_name: self.name.clone(),
                });
                err(warnings, errors)
            }
        }
    }
}

#[derive(Debug, Clone, Eq)]
pub struct TypedStructField {
    pub(crate) name: Ident,
    pub(crate) r#type: TypeId,
    pub(crate) span: Span,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl Hash for TypedStructField {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        look_up_type_id(self.r#type).hash(state);
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypedStructField {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && look_up_type_id(self.r#type) == look_up_type_id(other.r#type)
    }
}

impl CopyTypes for TypedStructField {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.r#type = match look_up_type_id(self.r#type).matches_type_parameter(type_mapping) {
            Some(matching_id) => insert_type(TypeInfo::Ref(matching_id)),
            None => insert_type(look_up_type_id_raw(self.r#type)),
        };
    }
}

impl TypedStructField {
    pub fn generate_json_abi(&self) -> Property {
        Property {
            name: self.name.to_string(),
            type_field: self.r#type.json_abi_str(),
            components: self.r#type.generate_json_abi(),
        }
    }
}
