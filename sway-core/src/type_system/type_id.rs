use std::fmt;
use sway_types::{JsonTypeApplication, JsonTypeDeclaration, Span, Spanned};

use crate::types::*;

use super::*;

/// A identifier to uniquely refer to our type terms
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct TypeId(usize);

impl std::ops::Deref for TypeId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&look_up_type_id(*self).to_string())
    }
}

impl fmt::Debug for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&look_up_type_id(*self).to_string())
    }
}

impl From<usize> for TypeId {
    fn from(o: usize) -> Self {
        TypeId(o)
    }
}

impl UnresolvedTypeCheck for TypeId {
    fn check_for_unresolved_types(&self) -> Vec<CompileError> {
        use TypeInfo::*;
        let span_override = if let TypeInfo::Ref(_, span) = look_up_type_id_raw(*self) {
            Some(span)
        } else {
            None
        };
        match look_up_type_id(*self) {
            UnknownGeneric { name } => vec![CompileError::UnableToInferGeneric {
                ty: name.as_str().to_string(),
                span: span_override.unwrap_or_else(|| name.span()),
            }],
            _ => vec![],
        }
    }
}

impl JsonAbiString for TypeId {
    fn json_abi_str(&self) -> String {
        look_up_type_id(*self).json_abi_str()
    }
}

impl ToJsonAbi for TypeId {
    type Output = Option<Vec<Property>>;

    fn generate_json_abi(&self) -> Self::Output {
        match look_up_type_id(*self) {
            TypeInfo::Array(type_id, _, _) => Some(vec![Property {
                name: "__array_element".to_string(),
                type_field: type_id.json_abi_str(),
                components: type_id.generate_json_abi(),
                type_arguments: type_id
                    .get_type_parameters()
                    .map(|v| v.iter().map(TypeParameter::generate_json_abi).collect()),
            }]),
            TypeInfo::Enum { variant_types, .. } => Some(
                variant_types
                    .iter()
                    .map(|x| x.generate_json_abi())
                    .collect(),
            ),
            TypeInfo::Struct { fields, .. } => {
                Some(fields.iter().map(|x| x.generate_json_abi()).collect())
            }
            TypeInfo::Tuple(fields) => Some(fields.iter().map(|x| x.generate_json_abi()).collect()),
            _ => None,
        }
    }
}

impl ReplaceSelfType for TypeId {
    fn replace_self_type(&mut self, self_type: TypeId) {
        match look_up_type_id(*self) {
            TypeInfo::SelfType => {
                *self = self_type;
            }
            TypeInfo::Enum {
                mut type_parameters,
                mut variant_types,
                ..
            } => {
                for type_parameter in type_parameters.iter_mut() {
                    type_parameter.replace_self_type(self_type);
                }
                for variant_type in variant_types.iter_mut() {
                    variant_type.replace_self_type(self_type);
                }
            }
            TypeInfo::Struct {
                mut type_parameters,
                mut fields,
                ..
            } => {
                for type_parameter in type_parameters.iter_mut() {
                    type_parameter.replace_self_type(self_type);
                }
                for field in fields.iter_mut() {
                    field.replace_self_type(self_type);
                }
            }
            TypeInfo::Ref(mut type_id, _) => {
                type_id.replace_self_type(self_type);
            }
            TypeInfo::Tuple(mut type_arguments) => {
                for type_argument in type_arguments.iter_mut() {
                    type_argument.replace_self_type(self_type);
                }
            }
            TypeInfo::Custom { type_arguments, .. } => {
                if let Some(mut type_arguments) = type_arguments {
                    for type_argument in type_arguments.iter_mut() {
                        type_argument.replace_self_type(self_type);
                    }
                }
            }
            TypeInfo::Array(mut type_id, _, _) => {
                type_id.replace_self_type(self_type);
            }
            TypeInfo::Storage { mut fields } => {
                for field in fields.iter_mut() {
                    field.replace_self_type(self_type);
                }
            }
            TypeInfo::Unknown
            | TypeInfo::UnknownGeneric { .. }
            | TypeInfo::Str(_)
            | TypeInfo::UnsignedInteger(_)
            | TypeInfo::Boolean
            | TypeInfo::ContractCaller { .. }
            | TypeInfo::Byte
            | TypeInfo::B256
            | TypeInfo::Numeric
            | TypeInfo::Contract
            | TypeInfo::ErrorRecovery => {}
        }
    }
}

impl TypeId {
    pub(crate) fn update_type(&mut self, type_mapping: &TypeMapping, span: &Span) {
        *self = match look_up_type_id(*self).matches_type_parameter(type_mapping) {
            Some(matching_id) => insert_type(TypeInfo::Ref(matching_id, span.clone())),
            None => {
                let ty = TypeInfo::Ref(insert_type(look_up_type_id_raw(*self)), span.clone());
                insert_type(ty)
            }
        };
    }

    pub(crate) fn get_type_parameters(&self) -> Option<Vec<TypeParameter>> {
        match look_up_type_id(*self) {
            TypeInfo::Enum {
                type_parameters, ..
            } => (!type_parameters.is_empty()).then(|| type_parameters),
            TypeInfo::Struct {
                type_parameters, ..
            } => (!type_parameters.is_empty()).then(|| type_parameters),
            _ => None,
        }
    }

    /// Indicates of a given type is generic or not. Rely on whether the type is `Custom` and
    /// consider the special case where the resolved type is a struct or enum with a name that
    /// matches the name of the `Custom`.
    pub(crate) fn is_generic_parameter(self, resolved_type_id: TypeId) -> bool {
        match (look_up_type_id(self), look_up_type_id(resolved_type_id)) {
            (
                TypeInfo::Custom { name, .. },
                TypeInfo::Enum {
                    name: enum_name, ..
                },
            ) => name != enum_name,
            (
                TypeInfo::Custom { name, .. },
                TypeInfo::Struct {
                    name: struct_name, ..
                },
            ) => name != struct_name,
            (TypeInfo::Custom { .. }, _) => true,
            _ => false,
        }
    }

    /// Return the components of a given (potentially generic) type while considering what it
    /// actually resolves to. These components are essentially of type of `JsonTypeApplication`.
    /// The method below also updates the provided list of `JsonTypeDeclaration`s  to add the newly
    /// discovered types.
    pub(crate) fn get_json_type_components(
        &self,
        types: &mut Vec<JsonTypeDeclaration>,
        resolved_type_id: TypeId,
    ) -> Option<Vec<JsonTypeApplication>> {
        match look_up_type_id(*self) {
            TypeInfo::Enum { variant_types, .. } => {
                // A list of all `JsonTypeDeclaration`s needed for the enum variants
                let variants = variant_types
                    .iter()
                    .map(|x| JsonTypeDeclaration {
                        type_id: *x.initial_type_id,
                        type_field: x.initial_type_id.get_json_type_str(x.type_id),
                        components: x.initial_type_id.get_json_type_components(types, x.type_id),
                        type_parameters: x
                            .initial_type_id
                            .get_json_type_parameters(types, x.type_id),
                    })
                    .collect::<Vec<_>>();
                types.extend(variants);

                // Generate the JSON data for the enum. This is basically a list of
                // `JsonTypeApplication`s
                Some(
                    variant_types
                        .iter()
                        .map(|x| JsonTypeApplication {
                            name: x.name.to_string(),
                            type_id: *x.initial_type_id,
                            type_arguments: x
                                .initial_type_id
                                .get_json_type_arguments(types, x.type_id),
                        })
                        .collect(),
                )
            }
            TypeInfo::Struct { fields, .. } => {
                // A list of all `JsonTypeDeclaration`s needed for the struct fields
                let field_types = fields
                    .iter()
                    .map(|x| JsonTypeDeclaration {
                        type_id: *x.initial_type_id,
                        type_field: x.initial_type_id.get_json_type_str(x.type_id),
                        components: x.initial_type_id.get_json_type_components(types, x.type_id),
                        type_parameters: x
                            .initial_type_id
                            .get_json_type_parameters(types, x.type_id),
                    })
                    .collect::<Vec<_>>();
                types.extend(field_types);

                // Generate the JSON data for the struct. This is basically a list of
                // `JsonTypeApplication`s
                Some(
                    fields
                        .iter()
                        .map(|x| JsonTypeApplication {
                            name: x.name.to_string(),
                            type_id: *x.initial_type_id,
                            type_arguments: x
                                .initial_type_id
                                .get_json_type_arguments(types, x.type_id),
                        })
                        .collect(),
                )
            }
            TypeInfo::Array(..) => {
                if let TypeInfo::Array(type_id, _, initial_type_id) =
                    look_up_type_id(resolved_type_id)
                {
                    // The `JsonTypeDeclaration`s needed for the array element type
                    let elem_ty = JsonTypeDeclaration {
                        type_id: *initial_type_id,
                        type_field: initial_type_id.get_json_type_str(type_id),
                        components: initial_type_id.get_json_type_components(types, type_id),
                        type_parameters: initial_type_id.get_json_type_parameters(types, type_id),
                    };
                    types.push(elem_ty);

                    // Generate the JSON data for the array. This is basically a single
                    // `JsonTypeApplication` for the array element type
                    Some(vec![JsonTypeApplication {
                        name: "__array_element".to_string(),
                        type_id: *initial_type_id,
                        type_arguments: initial_type_id.get_json_type_arguments(types, type_id),
                    }])
                } else {
                    unreachable!();
                }
            }
            TypeInfo::Tuple(_) => {
                if let TypeInfo::Tuple(fields) = look_up_type_id(resolved_type_id) {
                    // A list of all `JsonTypeDeclaration`s needed for the tuple fields
                    let fields_types = fields
                        .iter()
                        .map(|x| JsonTypeDeclaration {
                            type_id: *x.initial_type_id,
                            type_field: x.initial_type_id.get_json_type_str(x.type_id),
                            components: x
                                .initial_type_id
                                .get_json_type_components(types, x.type_id),
                            type_parameters: x
                                .initial_type_id
                                .get_json_type_parameters(types, x.type_id),
                        })
                        .collect::<Vec<_>>();
                    types.extend(fields_types);

                    // Generate the JSON data for the tuple. This is basically a list of
                    // `JsonTypeApplication`s
                    Some(
                        fields
                            .iter()
                            .map(|x| JsonTypeApplication {
                                name: "__tuple_element".to_string(),
                                type_id: *x.initial_type_id,
                                type_arguments: x
                                    .initial_type_id
                                    .get_json_type_arguments(types, x.type_id),
                            })
                            .collect(),
                    )
                } else {
                    unreachable!()
                }
            }
            TypeInfo::Custom { type_arguments, .. } => {
                if !self.is_generic_parameter(resolved_type_id) {
                    // A list of all `JsonTypeDeclaration`s needed for the type arguments
                    let type_args = type_arguments
                        .unwrap_or_default()
                        .iter()
                        .zip(
                            resolved_type_id
                                .get_type_parameters()
                                .unwrap_or_default()
                                .iter(),
                        )
                        .map(|(v, p)| JsonTypeDeclaration {
                            type_id: *v.initial_type_id,
                            type_field: v.initial_type_id.get_json_type_str(p.type_id),
                            components: v
                                .initial_type_id
                                .get_json_type_components(types, p.type_id),
                            type_parameters: v
                                .initial_type_id
                                .get_json_type_parameters(types, p.type_id),
                        })
                        .collect::<Vec<_>>();
                    types.extend(type_args);

                    resolved_type_id.get_json_type_components(types, resolved_type_id)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Return the type parameters of a given (potentially generic) type while considering what it
    /// actually resolves to. These parameters are essentially of type of `usize` which are
    /// basically the IDs of some set of `JsonTypeDeclaration`s. The method below also updates the
    /// provide list of `JsonTypeDeclaration`s  to add the newly discovered types.
    pub(crate) fn get_json_type_parameters(
        &self,
        types: &mut Vec<JsonTypeDeclaration>,
        resolved_type_id: TypeId,
    ) -> Option<Vec<usize>> {
        match self.is_generic_parameter(resolved_type_id) {
            true => None,
            false => resolved_type_id.get_type_parameters().map(|v| {
                v.iter()
                    .map(|v| v.get_json_type_parameter(types))
                    .collect::<Vec<_>>()
            }),
        }
    }

    /// Return the type arguments of a given (potentially generic) type while considering what it
    /// actually resolves to. These arguments are essentially of type of `JsonTypeApplication`. The
    /// method below also updates the provided list of `JsonTypeDeclaration`s  to add the newly
    /// discovered types.
    pub(crate) fn get_json_type_arguments(
        &self,
        types: &mut Vec<JsonTypeDeclaration>,
        resolved_type_id: TypeId,
    ) -> Option<Vec<JsonTypeApplication>> {
        let resolved_params = resolved_type_id.get_type_parameters();
        match look_up_type_id(*self) {
            TypeInfo::Custom {
                type_arguments: Some(type_arguments),
                ..
            } => (!type_arguments.is_empty()).then_some({
                let resolved_params = resolved_params.unwrap_or_default();
                let json_type_arguments = type_arguments
                    .iter()
                    .zip(resolved_params.iter())
                    .map(|(v, p)| JsonTypeDeclaration {
                        type_id: *v.initial_type_id,
                        type_field: v.initial_type_id.get_json_type_str(p.type_id),
                        components: v.initial_type_id.get_json_type_components(types, p.type_id),
                        type_parameters: v
                            .initial_type_id
                            .get_json_type_parameters(types, p.type_id),
                    })
                    .collect::<Vec<_>>();
                types.extend(json_type_arguments);

                type_arguments
                    .iter()
                    .map(|arg| JsonTypeApplication {
                        name: "".to_string(),
                        type_id: *arg.initial_type_id,
                        type_arguments: arg
                            .initial_type_id
                            .get_json_type_arguments(types, arg.type_id),
                    })
                    .collect::<Vec<_>>()
            }),
            _ => None,
        }
    }

    /// Gives back a string that represents the type, considering what it resolves to
    pub(crate) fn get_json_type_str(&self, resolved_type_id: TypeId) -> String {
        if self.is_generic_parameter(resolved_type_id) {
            format!("generic {}", look_up_type_id(*self).json_abi_str())
        } else {
            match (look_up_type_id(*self), look_up_type_id(resolved_type_id)) {
                (TypeInfo::Custom { .. }, TypeInfo::Struct { .. }) => {
                    format!("struct {}", look_up_type_id(*self).json_abi_str())
                }
                (TypeInfo::Custom { .. }, TypeInfo::Enum { .. }) => {
                    format!("enum {}", look_up_type_id(*self).json_abi_str())
                }
                (TypeInfo::Tuple(fields), TypeInfo::Tuple(resolved_fields)) => {
                    assert_eq!(fields.len(), resolved_fields.len());
                    let field_strs = fields
                        .iter()
                        .map(|_| "_".to_string())
                        .collect::<Vec<String>>();
                    format!("({})", field_strs.join(", "))
                }
                (TypeInfo::Array(_, count, _), TypeInfo::Array(_, resolved_count, _)) => {
                    assert_eq!(count, resolved_count);
                    format!("[_; {count}]")
                }
                (TypeInfo::Custom { .. }, _) => {
                    format!("generic {}", look_up_type_id(*self).json_abi_str())
                }
                _ => look_up_type_id(*self).json_abi_str(),
            }
        }
    }
}
