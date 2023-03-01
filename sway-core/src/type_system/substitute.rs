use std::{collections::BTreeMap, fmt};

use super::*;
use crate::engine_threading::*;

pub trait SubstTypes {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>);

    fn subst(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        if !type_mapping.is_empty() {
            self.subst_inner(type_mapping, engines);
        }
    }
}

type SourceType = TypeId;
type DestinationType = TypeId;

/// The [TypeSubstMap] is used to create a mapping between a [SourceType] (LHS)
/// and a [DestinationType] (RHS).
pub struct TypeSubstMap {
    mapping: BTreeMap<SourceType, DestinationType>,
}

impl DisplayWithEngines for TypeSubstMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: Engines<'_>) -> fmt::Result {
        write!(
            f,
            "TypeSubstMap {{ {} }}",
            self.mapping
                .iter()
                .map(|(source_type, dest_type)| {
                    format!(
                        "{} -> {}",
                        engines.help_out(source_type),
                        engines.help_out(dest_type)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl fmt::Debug for TypeSubstMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TypeSubstMap {{ {} }}",
            self.mapping
                .iter()
                .map(|(source_type, dest_type)| { format!("{source_type:?} -> {dest_type:?}") })
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl TypeSubstMap {
    /// Returns `true` if the [TypeSubstMap] is empty.
    pub(crate) fn is_empty(&self) -> bool {
        self.mapping.is_empty()
    }

    /// Constructs a new [TypeSubstMap] from a list of [TypeParameter]s
    /// `type_parameters`. The [SourceType]s of the resulting [TypeSubstMap] are
    /// the [TypeId]s from `type_parameters` and the [DestinationType]s are the
    /// new [TypeId]s created from a transformation upon `type_parameters`.
    pub(crate) fn from_type_parameters(
        engines: Engines<'_>,
        type_parameters: &[TypeParameter],
    ) -> TypeSubstMap {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        let mapping = type_parameters
            .iter()
            .map(|x| {
                (
                    x.type_id,
                    type_engine.insert(decl_engine, TypeInfo::Placeholder(x.clone())),
                )
            })
            .collect();
        TypeSubstMap { mapping }
    }

    /// Constructs a new [TypeSubstMap] from a superset [TypeId] and a subset
    /// [TypeId]. The [SourceType]s of the resulting [TypeSubstMap] are the
    /// [TypeId]s from `superset` and the [DestinationType]s are the [TypeId]s
    /// from `subset`. Thus, the resulting [TypeSubstMap] maps the type
    /// parameters of the superset [TypeId] to the type parameters of the subset
    /// [TypeId], and is used in monomorphization.
    ///
    /// *Importantly, this function does not check to see if the two types
    /// given are indeed a superset and subset of one another, but instead that
    /// is an assumption.*
    ///
    /// Here is an example, given these input types (in pseudo-code):
    ///
    /// ```ignore
    /// superset:
    ///
    /// TypeInfo::Struct {
    ///     name: "Either",
    ///     type_parameters: [L, R],
    ///     fields: ..
    /// }
    ///
    /// subset:
    ///
    /// TypeInfo::Struct {
    ///     name: "Either"
    ///     type_parameters: [u64, bool],
    ///     fields: ..
    /// }
    /// ```
    ///
    /// So then the resulting [TypeSubstMap] would look like:
    ///
    /// ```ignore
    /// TypeSubstMap {
    ///     mapping: [
    ///         (L, u64),
    ///         (R, bool)
    ///     ]
    /// }
    /// ````
    ///
    /// So, as we can see, the resulting [TypeSubstMap] is a mapping from the
    /// type parameters of the `superset` to the type parameters of the
    /// `subset`. This [TypeSubstMap] can be used to complete monomorphization on
    /// methods, etc, that are implemented for the type of `superset` so that
    /// they can be used for `subset`.
    pub(crate) fn from_superset_and_subset(
        type_engine: &TypeEngine,
        superset: TypeId,
        subset: TypeId,
    ) -> TypeSubstMap {
        match (type_engine.get(superset), type_engine.get(subset)) {
            (TypeInfo::UnknownGeneric { .. }, _) => TypeSubstMap {
                mapping: BTreeMap::from([(superset, subset)]),
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
                TypeSubstMap::from_superset_and_subset_helper(
                    type_engine,
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
                TypeSubstMap::from_superset_and_subset_helper(
                    type_engine,
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
                TypeSubstMap::from_superset_and_subset_helper(
                    type_engine,
                    type_parameters,
                    type_arguments,
                )
            }
            (TypeInfo::Tuple(type_parameters), TypeInfo::Tuple(type_arguments)) => {
                TypeSubstMap::from_superset_and_subset_helper(
                    type_engine,
                    type_parameters
                        .iter()
                        .map(|x| x.type_id)
                        .collect::<Vec<_>>(),
                    type_arguments.iter().map(|x| x.type_id).collect::<Vec<_>>(),
                )
            }
            (TypeInfo::Array(type_parameter, _), TypeInfo::Array(type_argument, _)) => {
                TypeSubstMap::from_superset_and_subset_helper(
                    type_engine,
                    vec![type_parameter.type_id],
                    vec![type_argument.type_id],
                )
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
                    .map(|x| x.type_argument.type_id)
                    .collect::<Vec<_>>();
                let type_arguments = type_arguments
                    .iter()
                    .map(|x| x.type_argument.type_id)
                    .collect::<Vec<_>>();
                TypeSubstMap::from_superset_and_subset_helper(
                    type_engine,
                    type_parameters,
                    type_arguments,
                )
            }
            (TypeInfo::Unknown, TypeInfo::Unknown)
            | (TypeInfo::Boolean, TypeInfo::Boolean)
            | (TypeInfo::SelfType, TypeInfo::SelfType)
            | (TypeInfo::B256, TypeInfo::B256)
            | (TypeInfo::Numeric, TypeInfo::Numeric)
            | (TypeInfo::Contract, TypeInfo::Contract)
            | (TypeInfo::ErrorRecovery, TypeInfo::ErrorRecovery)
            | (TypeInfo::Str(_), TypeInfo::Str(_))
            | (TypeInfo::UnsignedInteger(_), TypeInfo::UnsignedInteger(_))
            | (TypeInfo::ContractCaller { .. }, TypeInfo::ContractCaller { .. }) => TypeSubstMap {
                mapping: BTreeMap::new(),
            },
            _ => TypeSubstMap {
                mapping: BTreeMap::new(),
            },
        }
    }

    /// Constructs a [TypeSubstMap] from a list of [TypeId]s `type_parameters`
    /// and a list of [TypeId]s `type_arguments`, the generated [TypeSubstMap]
    /// is extended with the result from calling `from_superset_and_subset`
    /// with each [SourceType]s and [DestinationType]s in the original [TypeSubstMap].
    fn from_superset_and_subset_helper(
        type_engine: &TypeEngine,
        type_parameters: Vec<SourceType>,
        type_arguments: Vec<DestinationType>,
    ) -> TypeSubstMap {
        let mut type_mapping =
            TypeSubstMap::from_type_parameters_and_type_arguments(type_parameters, type_arguments);

        for (s, d) in type_mapping.mapping.clone().iter() {
            type_mapping.mapping.extend(
                TypeSubstMap::from_superset_and_subset(type_engine, *s, *d)
                    .mapping
                    .iter(),
            );
        }
        type_mapping
    }

    /// Constructs a [TypeSubstMap] from a list of [TypeId]s `type_parameters`
    /// and a list of [TypeId]s `type_arguments`. The [SourceType]s of the
    /// resulting [TypeSubstMap] are the [TypeId]s from `type_parameters` and the
    /// [DestinationType]s are the [TypeId]s from `type_arguments`.
    pub(crate) fn from_type_parameters_and_type_arguments(
        type_parameters: Vec<SourceType>,
        type_arguments: Vec<DestinationType>,
    ) -> TypeSubstMap {
        let mapping = type_parameters
            .into_iter()
            .zip(type_arguments.into_iter())
            .collect();
        TypeSubstMap { mapping }
    }

    /// Given a [TypeId] `type_id`, find (or create) a match for `type_id` in
    /// this [TypeSubstMap] and return it, if there is a match. Importantly, this
    /// function is recursive, so any `type_id` it's given will undergo
    /// recursive calls this function. For instance, in the case of
    /// [TypeInfo::Struct], both `fields` and `type_parameters` will recursively
    /// call `find_match` (via calling [SubstTypes]).
    ///
    /// A match can be found in two different circumstances:
    /// - `type_id` is a [TypeInfo::Custom] or [TypeInfo::UnknownGeneric]
    ///
    /// A match is potentially created (i.e. a new `TypeId` is created) in these
    /// circumstances:
    /// - `type_id` is a [TypeInfo::Struct], [TypeInfo::Enum],
    ///     [TypeInfo::Array], or [TypeInfo::Tuple] and one of the sub-types
    ///     finds a match in a recursive call to `find_match`
    ///
    /// A match cannot be found in any other circumstance.
    pub(crate) fn find_match(&self, type_id: TypeId, engines: Engines<'_>) -> Option<TypeId> {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        let type_info = type_engine.get(type_id);
        match type_info {
            TypeInfo::Custom { .. } => iter_for_match(engines, self, &type_info),
            TypeInfo::UnknownGeneric { .. } => iter_for_match(engines, self, &type_info),
            TypeInfo::Placeholder(_) => iter_for_match(engines, self, &type_info),
            TypeInfo::Struct {
                fields,
                call_path,
                type_parameters,
            } => {
                let mut need_to_create_new = false;
                let fields = fields
                    .into_iter()
                    .map(|mut field| {
                        if let Some(type_id) = self.find_match(field.type_argument.type_id, engines)
                        {
                            need_to_create_new = true;
                            field.type_argument.type_id = type_id;
                        }
                        field
                    })
                    .collect::<Vec<_>>();
                let type_parameters = type_parameters
                    .into_iter()
                    .map(|mut type_param| {
                        if let Some(type_id) = self.find_match(type_param.type_id, engines) {
                            need_to_create_new = true;
                            type_param.type_id = type_id;
                        }
                        type_param
                    })
                    .collect::<Vec<_>>();
                if need_to_create_new {
                    Some(type_engine.insert(
                        decl_engine,
                        TypeInfo::Struct {
                            fields,
                            call_path,
                            type_parameters,
                        },
                    ))
                } else {
                    None
                }
            }
            TypeInfo::Enum {
                variant_types,
                call_path,
                type_parameters,
            } => {
                let mut need_to_create_new = false;
                let variant_types = variant_types
                    .into_iter()
                    .map(|mut variant| {
                        if let Some(type_id) =
                            self.find_match(variant.type_argument.type_id, engines)
                        {
                            need_to_create_new = true;
                            variant.type_argument.type_id = type_id;
                        }
                        variant
                    })
                    .collect::<Vec<_>>();
                let type_parameters = type_parameters
                    .into_iter()
                    .map(|mut type_param| {
                        if let Some(type_id) = self.find_match(type_param.type_id, engines) {
                            need_to_create_new = true;
                            type_param.type_id = type_id;
                        }
                        type_param
                    })
                    .collect::<Vec<_>>();
                if need_to_create_new {
                    Some(type_engine.insert(
                        decl_engine,
                        TypeInfo::Enum {
                            variant_types,
                            type_parameters,
                            call_path,
                        },
                    ))
                } else {
                    None
                }
            }
            TypeInfo::Array(mut elem_ty, count) => {
                self.find_match(elem_ty.type_id, engines).map(|type_id| {
                    elem_ty.type_id = type_id;
                    type_engine.insert(decl_engine, TypeInfo::Array(elem_ty, count))
                })
            }
            TypeInfo::Tuple(fields) => {
                let mut need_to_create_new = false;
                let fields = fields
                    .into_iter()
                    .map(|mut field| {
                        if let Some(type_id) = self.find_match(field.type_id, engines) {
                            need_to_create_new = true;
                            field.type_id = type_id;
                        }
                        field
                    })
                    .collect::<Vec<_>>();
                if need_to_create_new {
                    Some(type_engine.insert(decl_engine, TypeInfo::Tuple(fields)))
                } else {
                    None
                }
            }
            TypeInfo::Storage { fields } => {
                let mut need_to_create_new = false;
                let fields = fields
                    .into_iter()
                    .map(|mut field| {
                        if let Some(type_id) = self.find_match(field.type_argument.type_id, engines)
                        {
                            need_to_create_new = true;
                            field.type_argument.type_id = type_id;
                        }
                        field
                    })
                    .collect::<Vec<_>>();
                if need_to_create_new {
                    Some(type_engine.insert(decl_engine, TypeInfo::Storage { fields }))
                } else {
                    None
                }
            }
            TypeInfo::Unknown
            | TypeInfo::Str(..)
            | TypeInfo::UnsignedInteger(..)
            | TypeInfo::Boolean
            | TypeInfo::ContractCaller { .. }
            | TypeInfo::SelfType
            | TypeInfo::B256
            | TypeInfo::Numeric
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::Contract
            | TypeInfo::ErrorRecovery => None,
        }
    }
}

fn iter_for_match(
    engines: Engines<'_>,
    type_mapping: &TypeSubstMap,
    type_info: &TypeInfo,
) -> Option<TypeId> {
    let type_engine = engines.te();
    for (source_type, dest_type) in type_mapping.mapping.iter() {
        if type_engine.get(*source_type).eq(type_info, engines) {
            return Some(*dest_type);
        }
    }
    None
}
