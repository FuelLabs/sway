use crate::{
    decl_engine::{DeclEngine, DeclEngineInsert},
    engine_threading::{
        DebugWithEngines, Engines, PartialEqWithEngines, PartialEqWithEnginesContext,
    },
    type_system::priv_prelude::*,
};
use std::{collections::BTreeMap, fmt};
use sway_types::Spanned;

type SourceType = TypeId;
type DestinationType = TypeId;

/// The [TypeSubstMap] is used to create a mapping between a [SourceType] (LHS)
/// and a [DestinationType] (RHS).
#[derive(Clone, Default)]
pub struct TypeSubstMap {
    mapping: BTreeMap<SourceType, DestinationType>,
}

impl DebugWithEngines for TypeSubstMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(
            f,
            "TypeSubstMap {{ {} }}",
            self.mapping
                .iter()
                .map(|(source_type, dest_type)| {
                    format!(
                        "{:?} -> {:?}",
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

    /// Constructs a new empty [TypeSubstMap].
    pub(crate) fn new() -> TypeSubstMap {
        TypeSubstMap {
            mapping: BTreeMap::<SourceType, DestinationType>::new(),
        }
    }

    /// Constructs a new [TypeSubstMap] from a list of [TypeParameter]s
    /// `type_parameters`. The [SourceType]s of the resulting [TypeSubstMap] are
    /// the [TypeId]s from `type_parameters` and the [DestinationType]s are the
    /// new [TypeId]s created from a transformation upon `type_parameters`.
    pub(crate) fn from_type_parameters(
        engines: &Engines,
        type_parameters: &[TypeParameter],
    ) -> TypeSubstMap {
        let type_engine = engines.te();
        let mapping = type_parameters
            .iter()
            .map(|x| {
                (
                    x.type_id,
                    type_engine.insert(
                        engines,
                        TypeInfo::Placeholder(x.clone()),
                        x.name_ident.span().source_id(),
                    ),
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
        decl_engine: &DeclEngine,
        superset: TypeId,
        subset: TypeId,
    ) -> TypeSubstMap {
        match (&*type_engine.get(superset), &*type_engine.get(subset)) {
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
                    .clone()
                    .unwrap_or_default()
                    .iter()
                    .map(|x| x.type_id)
                    .collect::<Vec<_>>();
                let type_arguments = type_arguments
                    .clone()
                    .unwrap_or_default()
                    .iter()
                    .map(|x| x.type_id)
                    .collect::<Vec<_>>();
                TypeSubstMap::from_superset_and_subset_helper(
                    type_engine,
                    decl_engine,
                    type_parameters,
                    type_arguments,
                )
            }
            (TypeInfo::Enum(decl_ref_params), TypeInfo::Enum(decl_ref_args)) => {
                let decl_params = decl_engine.get_enum(decl_ref_params);
                let decl_args = decl_engine.get_enum(decl_ref_args);
                let type_parameters = decl_params
                    .type_parameters
                    .iter()
                    .map(|x| x.type_id)
                    .collect::<Vec<_>>();
                let type_arguments = decl_args
                    .type_parameters
                    .iter()
                    .map(|x| x.type_id)
                    .collect::<Vec<_>>();
                TypeSubstMap::from_superset_and_subset_helper(
                    type_engine,
                    decl_engine,
                    type_parameters,
                    type_arguments,
                )
            }
            (TypeInfo::Struct(decl_ref_params), TypeInfo::Struct(decl_ref_args)) => {
                let decl_params = decl_engine.get_struct(decl_ref_params);
                let decl_args = decl_engine.get_struct(decl_ref_args);

                let type_parameters = decl_params
                    .type_parameters
                    .iter()
                    .map(|x| x.type_id)
                    .collect::<Vec<_>>();
                let type_arguments = decl_args
                    .type_parameters
                    .iter()
                    .map(|x| x.type_id)
                    .collect::<Vec<_>>();
                TypeSubstMap::from_superset_and_subset_helper(
                    type_engine,
                    decl_engine,
                    type_parameters,
                    type_arguments,
                )
            }
            (TypeInfo::Tuple(type_parameters), TypeInfo::Tuple(type_arguments)) => {
                TypeSubstMap::from_superset_and_subset_helper(
                    type_engine,
                    decl_engine,
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
                    decl_engine,
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
                    decl_engine,
                    type_parameters,
                    type_arguments,
                )
            }
            (TypeInfo::Unknown, TypeInfo::Unknown)
            | (TypeInfo::Boolean, TypeInfo::Boolean)
            | (TypeInfo::B256, TypeInfo::B256)
            | (TypeInfo::Numeric, TypeInfo::Numeric)
            | (TypeInfo::Contract, TypeInfo::Contract)
            | (TypeInfo::ErrorRecovery(_), TypeInfo::ErrorRecovery(_))
            | (TypeInfo::StringSlice, TypeInfo::StringSlice)
            | (TypeInfo::StringArray(_), TypeInfo::StringArray(_))
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
        decl_engine: &DeclEngine,
        type_parameters: Vec<SourceType>,
        type_arguments: Vec<DestinationType>,
    ) -> TypeSubstMap {
        let mut type_mapping =
            TypeSubstMap::from_type_parameters_and_type_arguments(type_parameters, type_arguments);

        for (s, d) in type_mapping.mapping.clone().iter() {
            type_mapping.mapping.extend(
                TypeSubstMap::from_superset_and_subset(type_engine, decl_engine, *s, *d)
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
        let mapping = type_parameters.into_iter().zip(type_arguments).collect();
        TypeSubstMap { mapping }
    }

    pub(crate) fn extend(&mut self, subst_map: &TypeSubstMap) {
        self.mapping.extend(subst_map.mapping.iter());
    }

    /// Given a [TypeId] `type_id`, find (or create) a match for `type_id` in
    /// this [TypeSubstMap] and return it, if there is a match. Importantly, this
    /// function is recursive, so any `type_id` it's given will undergo
    /// recursive calls of this function. For instance, in the case of
    /// [TypeInfo::Struct], both `fields` and `type_parameters` will recursively
    /// call `find_match` (via calling [SubstTypes]).
    ///
    /// A match can be found in these circumstances:
    /// - `type_id` is one of the following: [TypeInfo::Custom],
    ///   [TypeInfo::UnknownGeneric], [TypeInfo::Placeholder], or [TypeInfo::TraitType].
    ///
    /// A match is potentially created (i.e. a new [TypeId] is created) in these
    /// circumstances:
    /// - `type_id` is one of the following: [TypeInfo::Struct], [TypeInfo::Enum],
    ///    [TypeInfo::Array], [TypeInfo::Tuple], [TypeInfo::Storage], [TypeInfo::Alias],
    ///    [TypeInfo::Alias], [TypeInfo::Ptr], [TypeInfo::Slice], or [TypeInfo::Ref],
    ///    and one of the contained types (e.g. a struct field, or a referenced type)
    ///    finds a match in a recursive call to `find_match`.
    ///
    /// A match cannot be found in any other circumstance.
    pub(crate) fn find_match(&self, type_id: TypeId, engines: &Engines) -> Option<TypeId> {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        let type_info = type_engine.get(type_id);
        match (*type_info).clone() {
            TypeInfo::Custom { .. } => iter_for_match(engines, self, &type_info),
            TypeInfo::UnknownGeneric { .. } => iter_for_match(engines, self, &type_info),
            TypeInfo::Placeholder(_) => iter_for_match(engines, self, &type_info),
            TypeInfo::TypeParam(_) => None,
            TypeInfo::Struct(decl_ref) => {
                let mut decl = (*decl_engine.get_struct(&decl_ref)).clone();
                let mut need_to_create_new = false;
                for field in &mut decl.fields {
                    if let Some(type_id) = self.find_match(field.type_argument.type_id, engines) {
                        need_to_create_new = true;
                        field.type_argument.type_id = type_id;
                    }
                }
                for type_param in &mut decl.type_parameters {
                    if let Some(type_id) = self.find_match(type_param.type_id, engines) {
                        need_to_create_new = true;
                        type_param.type_id = type_id;
                    }
                }
                if need_to_create_new {
                    let new_decl_ref = decl_engine.insert(decl);
                    Some(type_engine.insert(
                        engines,
                        TypeInfo::Struct(new_decl_ref.clone()),
                        new_decl_ref.decl_span().source_id(),
                    ))
                } else {
                    None
                }
            }
            TypeInfo::Enum(decl_ref) => {
                let mut decl = (*decl_engine.get_enum(&decl_ref)).clone();
                let mut need_to_create_new = false;

                for variant in &mut decl.variants {
                    if let Some(type_id) = self.find_match(variant.type_argument.type_id, engines) {
                        need_to_create_new = true;
                        variant.type_argument.type_id = type_id;
                    }
                }

                for type_param in &mut decl.type_parameters {
                    if let Some(type_id) = self.find_match(type_param.type_id, engines) {
                        need_to_create_new = true;
                        type_param.type_id = type_id;
                    }
                }
                if need_to_create_new {
                    let new_decl_ref = decl_engine.insert(decl);
                    Some(type_engine.insert(
                        engines,
                        TypeInfo::Enum(new_decl_ref.clone()),
                        new_decl_ref.decl_span().source_id(),
                    ))
                } else {
                    None
                }
            }
            TypeInfo::Array(mut elem_ty, count) => {
                self.find_match(elem_ty.type_id, engines).map(|type_id| {
                    elem_ty.type_id = type_id;
                    type_engine.insert(
                        engines,
                        TypeInfo::Array(elem_ty.clone(), count.clone()),
                        elem_ty.span.source_id(),
                    )
                })
            }
            TypeInfo::Tuple(fields) => {
                let mut need_to_create_new = false;
                let mut source_id = None;
                let fields = fields
                    .into_iter()
                    .map(|mut field| {
                        if let Some(type_id) = self.find_match(field.type_id, engines) {
                            need_to_create_new = true;
                            source_id = field.span.source_id().cloned();
                            field.type_id = type_id;
                        }
                        field.clone()
                    })
                    .collect::<Vec<_>>();
                if need_to_create_new {
                    Some(type_engine.insert(engines, TypeInfo::Tuple(fields), source_id.as_ref()))
                } else {
                    None
                }
            }
            TypeInfo::Storage { fields } => {
                let mut need_to_create_new = false;
                let mut source_id = None;
                let fields = fields
                    .into_iter()
                    .map(|mut field| {
                        if let Some(type_id) = self.find_match(field.type_argument.type_id, engines)
                        {
                            need_to_create_new = true;
                            source_id = field.span.source_id().copied();
                            field.type_argument.type_id = type_id;
                        }
                        field.clone()
                    })
                    .collect::<Vec<_>>();
                if need_to_create_new {
                    Some(type_engine.insert(
                        engines,
                        TypeInfo::Storage { fields },
                        source_id.as_ref(),
                    ))
                } else {
                    None
                }
            }
            TypeInfo::Alias { name, mut ty } => {
                self.find_match(ty.type_id, engines).map(|type_id| {
                    ty.type_id = type_id;
                    type_engine.insert(
                        engines,
                        TypeInfo::Alias {
                            name: name.clone(),
                            ty: ty.clone(),
                        },
                        ty.span.source_id(),
                    )
                })
            }
            TypeInfo::Ptr(mut ty) => self.find_match(ty.type_id, engines).map(|type_id| {
                ty.type_id = type_id;
                type_engine.insert(engines, TypeInfo::Ptr(ty.clone()), ty.span.source_id())
            }),
            TypeInfo::Slice(mut ty) => self.find_match(ty.type_id, engines).map(|type_id| {
                ty.type_id = type_id;
                type_engine.insert(engines, TypeInfo::Slice(ty.clone()), ty.span.source_id())
            }),
            TypeInfo::TraitType { .. } => iter_for_match(engines, self, &type_info),
            TypeInfo::Ref {
                to_mutable_value,
                referenced_type: mut ty,
            } => self.find_match(ty.type_id, engines).map(|type_id| {
                ty.type_id = type_id;
                type_engine.insert(
                    engines,
                    TypeInfo::Ref {
                        to_mutable_value,
                        referenced_type: ty.clone(),
                    },
                    ty.span.source_id(),
                )
            }),
            TypeInfo::Unknown
            | TypeInfo::Never
            | TypeInfo::StringArray(..)
            | TypeInfo::StringSlice
            | TypeInfo::UnsignedInteger(..)
            | TypeInfo::Boolean
            | TypeInfo::ContractCaller { .. }
            | TypeInfo::B256
            | TypeInfo::Numeric
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::Contract
            | TypeInfo::ErrorRecovery(..) => None,
        }
    }
}

fn iter_for_match(
    engines: &Engines,
    type_mapping: &TypeSubstMap,
    type_info: &TypeInfo,
) -> Option<TypeId> {
    let type_engine = engines.te();
    for (source_type, dest_type) in &type_mapping.mapping {
        if type_engine
            .get(*source_type)
            .eq(type_info, &PartialEqWithEnginesContext::new(engines))
        {
            return Some(*dest_type);
        }
    }
    None
}
