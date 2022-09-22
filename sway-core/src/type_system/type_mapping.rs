use super::*;

/// The [TypeMapping] is used to create a mapping between a type that we are
/// looking for (LHS) and the corresponding type that we want to replace it with
/// if we find it (RHS).
pub(crate) struct TypeMapping {
    mapping: Vec<(TypeId, TypeId)>,
}

impl TypeMapping {
    pub(crate) fn from_type_parameters(type_parameters: &[TypeParameter]) -> TypeMapping {
        let mapping = type_parameters
            .iter()
            .map(|x| {
                (
                    x.type_id,
                    insert_type(TypeInfo::UnknownGeneric {
                        name: x.name_ident.clone(),
                    }),
                )
            })
            .collect();
        TypeMapping { mapping }
    }

    pub(crate) fn from_superset_and_subset(
        superset_type: TypeId,
        subset_type: TypeId,
    ) -> TypeMapping {
        match (look_up_type_id(superset_type), look_up_type_id(subset_type)) {
            (TypeInfo::Ref(superset_type, _), TypeInfo::Ref(subset_type, _)) => {
                TypeMapping::from_superset_and_subset(superset_type, subset_type)
            }
            (TypeInfo::Ref(superset_type, _), _) => {
                TypeMapping::from_superset_and_subset(superset_type, subset_type)
            }
            (_, TypeInfo::Ref(subset_type, _)) => {
                TypeMapping::from_superset_and_subset(superset_type, subset_type)
            }
            (TypeInfo::UnknownGeneric { .. }, _) => TypeMapping {
                mapping: vec![(superset_type, subset_type)],
            },
            (
                TypeInfo::Custom {
                    type_arguments: type_parameters,
                    ..
                },
                TypeInfo::Custom { type_arguments, .. },
            ) => {
                let type_parameters = type_parameters
                    .unwrap_or_default()
                    .iter()
                    .map(|x| x.type_id)
                    .collect::<Vec<_>>();
                let type_arguments = type_arguments
                    .unwrap_or_default()
                    .iter()
                    .map(|x| x.type_id)
                    .collect::<Vec<_>>();
                TypeMapping::from_type_parameters_and_type_arguments(
                    type_parameters,
                    type_arguments,
                )
            }
            (
                TypeInfo::Enum {
                    type_parameters, ..
                },
                TypeInfo::Enum {
                    type_parameters: type_arguments,
                    ..
                },
            ) => {
                let type_parameters = type_parameters
                    .iter()
                    .map(|x| x.type_id)
                    .collect::<Vec<_>>();
                let type_arguments = type_arguments.iter().map(|x| x.type_id).collect::<Vec<_>>();
                TypeMapping::from_type_parameters_and_type_arguments(
                    type_parameters,
                    type_arguments,
                )
            }
            (
                TypeInfo::Struct {
                    type_parameters, ..
                },
                TypeInfo::Struct {
                    type_parameters: type_arguments,
                    ..
                },
            ) => {
                let type_parameters = type_parameters
                    .iter()
                    .map(|x| x.type_id)
                    .collect::<Vec<_>>();
                let type_arguments = type_arguments.iter().map(|x| x.type_id).collect::<Vec<_>>();
                TypeMapping::from_type_parameters_and_type_arguments(
                    type_parameters,
                    type_arguments,
                )
            }
            (TypeInfo::Tuple(type_parameters), TypeInfo::Tuple(type_arguments)) => {
                TypeMapping::from_type_parameters_and_type_arguments(
                    type_parameters
                        .iter()
                        .map(|x| x.type_id)
                        .collect::<Vec<_>>(),
                    type_arguments.iter().map(|x| x.type_id).collect::<Vec<_>>(),
                )
            }
            (TypeInfo::Array(superset_type, _, _), TypeInfo::Array(subset_type, _, _)) => {
                TypeMapping {
                    mapping: vec![(superset_type, subset_type)],
                }
            }
            (
                TypeInfo::Storage {
                    fields: type_parameters,
                },
                TypeInfo::Storage {
                    fields: type_arguments,
                },
            ) => {
                let type_parameters = type_parameters
                    .iter()
                    .map(|x| x.type_id)
                    .collect::<Vec<_>>();
                let type_arguments = type_arguments.iter().map(|x| x.type_id).collect::<Vec<_>>();
                TypeMapping::from_type_parameters_and_type_arguments(
                    type_parameters,
                    type_arguments,
                )
            }
            (TypeInfo::Unknown, TypeInfo::Unknown)
            | (TypeInfo::Boolean, TypeInfo::Boolean)
            | (TypeInfo::SelfType, TypeInfo::SelfType)
            | (TypeInfo::Byte, TypeInfo::Byte)
            | (TypeInfo::B256, TypeInfo::B256)
            | (TypeInfo::Numeric, TypeInfo::Numeric)
            | (TypeInfo::Contract, TypeInfo::Contract)
            | (TypeInfo::ErrorRecovery, TypeInfo::ErrorRecovery)
            | (TypeInfo::Str(_), TypeInfo::Str(_))
            | (TypeInfo::UnsignedInteger(_), TypeInfo::UnsignedInteger(_))
            | (TypeInfo::ContractCaller { .. }, TypeInfo::ContractCaller { .. }) => {
                TypeMapping { mapping: vec![] }
            }
            _ => TypeMapping { mapping: vec![] },
        }
    }

    fn from_type_parameters_and_type_arguments(
        type_parameters: Vec<TypeId>,
        type_arguments: Vec<TypeId>,
    ) -> TypeMapping {
        let mapping = type_parameters
            .into_iter()
            .zip(type_arguments.into_iter())
            .collect::<Vec<_>>();
        TypeMapping { mapping }
    }

    pub(crate) fn find_match(&self, type_id: TypeId) -> Option<TypeId> {
        let type_info = look_up_type_id(type_id);
        match type_info {
            TypeInfo::Custom { .. } => iter_for_match(self, &type_info),
            TypeInfo::UnknownGeneric { .. } => iter_for_match(self, &type_info),
            TypeInfo::Struct {
                mut fields,
                name,
                mut type_parameters,
            } => {
                fields.iter_mut().for_each(|field| field.copy_types(self));
                type_parameters
                    .iter_mut()
                    .for_each(|type_param| type_param.copy_types(self));
                Some(insert_type(TypeInfo::Struct {
                    fields,
                    name,
                    type_parameters,
                }))
            }
            TypeInfo::Enum {
                mut variant_types,
                name,
                mut type_parameters,
            } => {
                variant_types
                    .iter_mut()
                    .for_each(|variant_type| variant_type.copy_types(self));
                type_parameters
                    .iter_mut()
                    .for_each(|type_param| type_param.copy_types(self));
                Some(insert_type(TypeInfo::Enum {
                    variant_types,
                    type_parameters,
                    name,
                }))
            }
            TypeInfo::Array(ary_ty_id, count, initial_elem_ty) => {
                self.find_match(ary_ty_id).map(|matching_id| {
                    insert_type(TypeInfo::Array(matching_id, count, initial_elem_ty))
                })
            }
            TypeInfo::Tuple(mut fields) => {
                fields.iter_mut().for_each(|field| field.copy_types(self));
                Some(insert_type(TypeInfo::Tuple(fields)))
            }
            TypeInfo::Unknown
            | TypeInfo::Str(..)
            | TypeInfo::UnsignedInteger(..)
            | TypeInfo::Boolean
            | TypeInfo::Ref(..)
            | TypeInfo::ContractCaller { .. }
            | TypeInfo::SelfType
            | TypeInfo::Byte
            | TypeInfo::B256
            | TypeInfo::Numeric
            | TypeInfo::Contract
            | TypeInfo::Storage { .. }
            | TypeInfo::ErrorRecovery => None,
        }
    }

    pub(crate) fn unify_with_type_arguments(
        &self,
        type_arguments: &[TypeArgument],
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        for ((_, type_param_type), type_argument) in self.mapping.iter().zip(type_arguments.iter())
        {
            let (mut new_warnings, new_errors) = unify(
                *type_param_type,
                type_argument.type_id,
                &type_argument.span,
                "Type argument is not assignable to generic type parameter.",
            );
            warnings.append(&mut new_warnings);
            errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
        }
        ok((), warnings, errors)
    }
}

fn iter_for_match(type_mapping: &TypeMapping, type_info: &TypeInfo) -> Option<TypeId> {
    for (param, ty_id) in type_mapping.mapping.iter() {
        if look_up_type_id(*param) == *type_info {
            return Some(*ty_id);
        }
    }
    None
}
