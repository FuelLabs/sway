use super::*;

pub(crate) type TypeMapping = Vec<(TypeId, TypeId)>;

pub(crate) fn insert_type_parameters(
    type_engine: &TypeEngine,
    type_parameters: &[TypeParameter],
) -> TypeMapping {
    type_parameters
        .iter()
        .map(|x| {
            (
                x.type_id,
                type_engine.insert_type(TypeInfo::UnknownGeneric {
                    name: x.name_ident.clone(),
                }),
            )
        })
        .collect()
}

pub(crate) fn create_type_mapping(superset_type: TypeId, subset_type: TypeId) -> TypeMapping {
    match (look_up_type_id(superset_type), look_up_type_id(subset_type)) {
        (TypeInfo::Ref(superset_type, _), TypeInfo::Ref(subset_type, _)) => {
            create_type_mapping(superset_type, subset_type)
        }
        (TypeInfo::Ref(superset_type, _), _) => create_type_mapping(superset_type, subset_type),
        (_, TypeInfo::Ref(subset_type, _)) => create_type_mapping(superset_type, subset_type),
        (TypeInfo::UnknownGeneric { .. }, _) => {
            vec![(superset_type, subset_type)]
        }
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
            insert_type_parameters_with_type_arguments(type_parameters, type_arguments)
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
            insert_type_parameters_with_type_arguments(type_parameters, type_arguments)
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
            insert_type_parameters_with_type_arguments(type_parameters, type_arguments)
        }
        (TypeInfo::Tuple(type_parameters), TypeInfo::Tuple(type_arguments)) => {
            insert_type_parameters_with_type_arguments(
                type_parameters
                    .iter()
                    .map(|x| x.type_id)
                    .collect::<Vec<_>>(),
                type_arguments.iter().map(|x| x.type_id).collect::<Vec<_>>(),
            )
        }
        (TypeInfo::Array(superset_type, _), TypeInfo::Array(subset_type, _)) => {
            vec![(superset_type, subset_type)]
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
            insert_type_parameters_with_type_arguments(type_parameters, type_arguments)
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
        | (TypeInfo::ContractCaller { .. }, TypeInfo::ContractCaller { .. }) => vec![],
        _ => vec![],
    }
}

fn insert_type_parameters_with_type_arguments(
    type_parameters: Vec<TypeId>,
    type_arguments: Vec<TypeId>,
) -> TypeMapping {
    type_parameters
        .into_iter()
        .zip(type_arguments.into_iter())
        .collect::<Vec<_>>()
}
