use crate::{
    ast_elements::type_parameter::ConstGenericExpr,
    decl_engine::{DeclEngineGetParsedDeclId, DeclEngineInsert, ParsedDeclEngineInsert},
    engine_threading::{
        DebugWithEngines, Engines, PartialEqWithEngines, PartialEqWithEnginesContext,
    },
    type_system::priv_prelude::*,
};
use std::{collections::BTreeMap, fmt};

type SourceType = TypeId;
type DestinationType = TypeId;

/// The [TypeSubstMap] is used to create a mapping between a [SourceType] (LHS)
/// and a [DestinationType] (RHS).
#[derive(Clone, Default)]
pub struct TypeSubstMap {
    mapping: BTreeMap<SourceType, DestinationType>,
    pub const_generics_materialization: BTreeMap<String, crate::language::ty::TyExpression>,
}

impl DebugWithEngines for TypeSubstMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(
            f,
            "TypeSubstMap {{ {}; {} }}",
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
                .join(", "),
            self.const_generics_materialization
                .iter()
                .map(|(k, v)| {
                    format!("{:?} -> {:?}", k, engines.help_out(v))
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
            const_generics_materialization: BTreeMap::new(),
        }
    }

    pub(crate) fn source_ids_contains_concrete_type(&self, engines: &Engines) -> bool {
        for source_id in self.mapping.keys() {
            if source_id.is_concrete(engines, TreatNumericAs::Concrete) {
                return true;
            }
        }
        false
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
            .filter_map(|p| p.as_type_parameter())
            .filter(|p| {
                let type_info = type_engine.get(p.type_id);
                !matches!(*type_info, TypeInfo::Placeholder(_))
            })
            .map(|p| {
                (
                    p.type_id,
                    type_engine.new_placeholder(TypeParameter::Type(p.clone())),
                )
            })
            .collect();
        TypeSubstMap {
            mapping,
            const_generics_materialization: BTreeMap::new(),
        }
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
        engines: &Engines,
        superset: TypeId,
        subset: TypeId,
    ) -> TypeSubstMap {
        let type_engine = engines.te();
        let decl_engine = engines.de();

        match (&*type_engine.get(superset), &*type_engine.get(subset)) {
            (TypeInfo::UnknownGeneric { .. }, _) => TypeSubstMap {
                mapping: BTreeMap::from([(superset, subset)]),
                const_generics_materialization: BTreeMap::new(),
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
                    .map(|x| x.type_id())
                    .collect::<Vec<_>>();
                let type_arguments = type_arguments
                    .clone()
                    .unwrap_or_default()
                    .iter()
                    .map(|x| x.type_id())
                    .collect::<Vec<_>>();
                TypeSubstMap::from_superset_and_subset_helper(
                    engines,
                    type_parameters,
                    type_arguments,
                )
            }
            (TypeInfo::Enum(decl_ref_params), TypeInfo::Enum(decl_ref_args)) => {
                let decl_params = decl_engine.get_enum(decl_ref_params);
                let decl_args = decl_engine.get_enum(decl_ref_args);
                let type_parameters = decl_params
                    .generic_parameters
                    .iter()
                    .map(|x| {
                        let x = x
                            .as_type_parameter()
                            .expect("will only work with type parameters");
                        x.type_id
                    })
                    .collect::<Vec<_>>();
                let type_arguments = decl_args
                    .generic_parameters
                    .iter()
                    .map(|x| {
                        let x = x
                            .as_type_parameter()
                            .expect("will only work with type parameters");
                        x.type_id
                    })
                    .collect::<Vec<_>>();
                TypeSubstMap::from_superset_and_subset_helper(
                    engines,
                    type_parameters,
                    type_arguments,
                )
            }
            (TypeInfo::Struct(decl_ref_params), TypeInfo::Struct(decl_ref_args)) => {
                let decl_params = decl_engine.get_struct(decl_ref_params);
                let decl_args = decl_engine.get_struct(decl_ref_args);

                let type_parameters = decl_params
                    .generic_parameters
                    .iter()
                    .map(|x| {
                        let x = x
                            .as_type_parameter()
                            .expect("only works with type parameters");
                        x.type_id
                    })
                    .collect::<Vec<_>>();
                let type_arguments = decl_args
                    .generic_parameters
                    .iter()
                    .map(|x| {
                        let x = x
                            .as_type_parameter()
                            .expect("only works with type parameters");
                        x.type_id
                    })
                    .collect::<Vec<_>>();
                TypeSubstMap::from_superset_and_subset_helper(
                    engines,
                    type_parameters,
                    type_arguments,
                )
            }
            (TypeInfo::Tuple(type_parameters), TypeInfo::Tuple(type_arguments)) => {
                TypeSubstMap::from_superset_and_subset_helper(
                    engines,
                    type_parameters
                        .iter()
                        .map(|x| x.type_id())
                        .collect::<Vec<_>>(),
                    type_arguments
                        .iter()
                        .map(|x| x.type_id())
                        .collect::<Vec<_>>(),
                )
            }
            (TypeInfo::Array(type_parameter, l), TypeInfo::Array(type_argument, r)) => {
                let mut map = TypeSubstMap::from_superset_and_subset_helper(
                    engines,
                    vec![type_parameter.type_id()],
                    vec![type_argument.type_id()],
                );
                match (&l.expr(), &r.expr()) {
                    (
                        ConstGenericExpr::AmbiguousVariableExpression { ident },
                        ConstGenericExpr::Literal { val, .. },
                    ) => {
                        map.const_generics_materialization.insert(
                            ident.as_str().into(),
                            crate::language::ty::TyExpression {
                                expression: crate::language::ty::TyExpressionVariant::Literal(
                                    crate::language::Literal::U64(*val as u64),
                                ),
                                return_type: type_engine.id_of_u64(),
                                span: sway_types::Span::dummy(),
                            },
                        );
                        map
                    }
                    _ => map,
                }
            }
            (TypeInfo::Slice(type_parameter), TypeInfo::Slice(type_argument)) => {
                TypeSubstMap::from_superset_and_subset_helper(
                    engines,
                    vec![type_parameter.type_id()],
                    vec![type_argument.type_id()],
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
                const_generics_materialization: BTreeMap::new(),
            },
            _ => TypeSubstMap {
                mapping: BTreeMap::new(),
                const_generics_materialization: BTreeMap::new(),
            },
        }
    }

    /// Constructs a [TypeSubstMap] from a list of [TypeId]s `type_parameters`
    /// and a list of [TypeId]s `type_arguments`, the generated [TypeSubstMap]
    /// is extended with the result from calling `from_superset_and_subset`
    /// with each [SourceType]s and [DestinationType]s in the original [TypeSubstMap].
    fn from_superset_and_subset_helper(
        engines: &Engines,
        type_parameters: Vec<SourceType>,
        type_arguments: Vec<DestinationType>,
    ) -> TypeSubstMap {
        let mut type_mapping =
            TypeSubstMap::from_type_parameters_and_type_arguments(type_parameters, type_arguments);

        for (s, d) in type_mapping.mapping.clone().iter() {
            type_mapping.mapping.extend(
                TypeSubstMap::from_superset_and_subset(engines, *s, *d)
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
        TypeSubstMap {
            mapping,
            const_generics_materialization: BTreeMap::new(),
        }
    }

    pub(crate) fn from_type_parameters_and_type_arguments_and_const_generics(
        type_parameters: Vec<SourceType>,
        type_arguments: Vec<DestinationType>,
        const_generics_materialization: BTreeMap<String, crate::language::ty::TyExpression>,
    ) -> TypeSubstMap {
        let mapping = type_parameters.into_iter().zip(type_arguments).collect();
        TypeSubstMap {
            mapping,
            const_generics_materialization,
        }
    }

    pub(crate) fn extend(&mut self, other: &TypeSubstMap) {
        self.mapping.extend(other.mapping.iter());
        self.const_generics_materialization.extend(
            other
                .const_generics_materialization
                .iter()
                .map(|x| (x.0.clone(), x.1.clone())),
        );
    }

    pub(crate) fn insert(&mut self, source: SourceType, destination: DestinationType) {
        self.mapping.insert(source, destination);
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
    ///   [TypeInfo::Array], [TypeInfo::Tuple], [TypeInfo::Alias], [TypeInfo::Ptr],
    ///   [TypeInfo::Slice], or [TypeInfo::Ref],
    /// - and one of the contained types (e.g. a struct field, or a referenced type)
    ///   finds a match in a recursive call to `find_match`.
    ///
    /// A match cannot be found in any other circumstance.
    pub(crate) fn find_match(&self, type_id: TypeId, engines: &Engines) -> Option<TypeId> {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        let parsed_decl_engine = engines.pe();
        let type_info = type_engine.get(type_id);
        match (*type_info).clone() {
            TypeInfo::Custom { .. } => iter_for_match(engines, self, &type_info),
            TypeInfo::UnknownGeneric { .. } => iter_for_match(engines, self, &type_info),
            TypeInfo::Placeholder(_) => iter_for_match(engines, self, &type_info),
            TypeInfo::TypeParam(_) => None,
            TypeInfo::UntypedEnum(decl_id) => {
                let mut decl = (*parsed_decl_engine.get_enum(&decl_id)).clone();
                let mut need_to_create_new = false;

                for variant in &mut decl.variants {
                    if let Some(type_id) = self.find_match(variant.type_argument.type_id(), engines)
                    {
                        need_to_create_new = true;
                        *variant.type_argument.type_id_mut() = type_id;
                    }
                }

                for type_param in &mut decl.type_parameters {
                    let type_param = type_param
                        .as_type_parameter_mut()
                        .expect("only works with type parameters");
                    if let Some(type_id) = self.find_match(type_param.type_id, engines) {
                        need_to_create_new = true;
                        type_param.type_id = type_id;
                    }
                }
                if need_to_create_new {
                    let source_id = decl.span.source_id().copied();
                    let new_decl_id = engines.pe().insert(decl);
                    Some(type_engine.insert(
                        engines,
                        TypeInfo::UntypedEnum(new_decl_id),
                        source_id.as_ref(),
                    ))
                } else {
                    None
                }
            }
            TypeInfo::UntypedStruct(decl_id) => {
                let mut decl = (*parsed_decl_engine.get_struct(&decl_id)).clone();
                let mut need_to_create_new = false;
                for field in &mut decl.fields {
                    if let Some(type_id) = self.find_match(field.type_argument.type_id(), engines) {
                        need_to_create_new = true;
                        *field.type_argument.type_id_mut() = type_id;
                    }
                }
                for type_param in &mut decl.type_parameters {
                    let type_param = type_param
                        .as_type_parameter_mut()
                        .expect("only works with type parameters");
                    if let Some(type_id) = self.find_match(type_param.type_id, engines) {
                        need_to_create_new = true;
                        type_param.type_id = type_id;
                    }
                }
                if need_to_create_new {
                    let source_id = decl.span.source_id().copied();
                    let new_decl_id = parsed_decl_engine.insert(decl);
                    Some(type_engine.insert(
                        engines,
                        TypeInfo::UntypedStruct(new_decl_id),
                        source_id.as_ref(),
                    ))
                } else {
                    None
                }
            }
            TypeInfo::Struct(decl_id) => {
                let mut decl = (*decl_engine.get_struct(&decl_id)).clone();
                let mut need_to_create_new = false;
                for field in &mut decl.fields {
                    if let Some(type_id) = self.find_match(field.type_argument.type_id(), engines) {
                        need_to_create_new = true;
                        *field.type_argument.type_id_mut() = type_id;
                    }
                }
                for type_param in &mut decl
                    .generic_parameters
                    .iter_mut()
                    .filter_map(|x| x.as_type_parameter_mut())
                {
                    if let Some(type_id) = self.find_match(type_param.type_id, engines) {
                        need_to_create_new = true;
                        type_param.type_id = type_id;
                    }
                }
                if need_to_create_new {
                    let new_decl_ref =
                        decl_engine.insert(decl, decl_engine.get_parsed_decl_id(&decl_id).as_ref());
                    Some(type_engine.insert_struct(engines, *new_decl_ref.id()))
                } else {
                    None
                }
            }
            TypeInfo::Enum(decl_id) => {
                let mut decl = (*decl_engine.get_enum(&decl_id)).clone();
                let mut need_to_create_new = false;

                for variant in &mut decl.variants {
                    if let Some(type_id) = self.find_match(variant.type_argument.type_id(), engines)
                    {
                        need_to_create_new = true;
                        *variant.type_argument.type_id_mut() = type_id;
                    }
                }

                for type_param in &mut decl.generic_parameters {
                    let Some(type_param) = type_param.as_type_parameter_mut() else {
                        continue;
                    };
                    if let Some(type_id) = self.find_match(type_param.type_id, engines) {
                        need_to_create_new = true;
                        type_param.type_id = type_id;
                    }
                }
                if need_to_create_new {
                    let new_decl_ref =
                        decl_engine.insert(decl, decl_engine.get_parsed_decl_id(&decl_id).as_ref());
                    Some(type_engine.insert_enum(engines, *new_decl_ref.id()))
                } else {
                    None
                }
            }
            TypeInfo::Array(mut elem_type, length) => self
                .find_match(elem_type.type_id(), engines)
                .map(|type_id| {
                    *elem_type.type_id_mut() = type_id;
                    type_engine.insert_array(engines, elem_type, length)
                }),
            TypeInfo::Slice(mut elem_type) => {
                self.find_match(elem_type.type_id(), engines)
                    .map(|type_id| {
                        *elem_type.type_id_mut() = type_id;
                        type_engine.insert_slice(engines, elem_type)
                    })
            }
            TypeInfo::Tuple(fields) => {
                let mut need_to_create_new = false;
                let fields = fields
                    .into_iter()
                    .map(|mut field| {
                        if let Some(type_id) = self.find_match(field.type_id(), engines) {
                            need_to_create_new = true;
                            *field.type_id_mut() = type_id;
                        }
                        field.clone()
                    })
                    .collect::<Vec<_>>();
                if need_to_create_new {
                    Some(type_engine.insert_tuple(engines, fields))
                } else {
                    None
                }
            }
            TypeInfo::Alias { name, mut ty } => {
                self.find_match(ty.type_id(), engines).map(|type_id| {
                    *ty.type_id_mut() = type_id;
                    type_engine.new_alias(engines, name, ty)
                })
            }
            TypeInfo::Ptr(mut ty) => self.find_match(ty.type_id(), engines).map(|type_id| {
                *ty.type_id_mut() = type_id;
                type_engine.insert_ptr(engines, ty)
            }),
            TypeInfo::TraitType { .. } => iter_for_match(engines, self, &type_info),
            TypeInfo::Ref {
                to_mutable_value,
                referenced_type: mut ty,
            } => self.find_match(ty.type_id(), engines).map(|type_id| {
                *ty.type_id_mut() = type_id;
                type_engine.insert_ref(engines, to_mutable_value, ty)
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
        let source_type_info = type_engine.get(*source_type);

        // Allows current placeholder(T:T1+T2) to match source placeholder(T:T1)
        if let (
            TypeInfo::Placeholder(source_type_param),
            TypeInfo::Placeholder(current_type_param),
        ) = ((*source_type_info).clone(), type_info)
        {
            let source_type_param = source_type_param
                .as_type_parameter()
                .expect("only works with type parameters");
            let current_type_param = current_type_param
                .as_type_parameter()
                .expect("only works with type parameters");
            if source_type_param.name.as_str() == current_type_param.name.as_str()
                && current_type_param
                    .trait_constraints
                    .iter()
                    .all(|current_tc| {
                        source_type_param.trait_constraints.iter().any(|source_tc| {
                            source_tc.eq(current_tc, &PartialEqWithEnginesContext::new(engines))
                        })
                    })
            {
                return Some(*dest_type);
            }
        }

        if source_type_info.eq(type_info, &PartialEqWithEnginesContext::new(engines)) {
            return Some(*dest_type);
        }
    }

    None
}
