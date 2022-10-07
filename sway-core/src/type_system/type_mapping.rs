use super::*;

type SourceType = TypeId;
type DestinationType = TypeId;

/// The [TypeMapping] is used to create a mapping between a [SourceType] (LHS)
/// and a [DestinationType] (RHS).
pub(crate) struct TypeMapping {
    mapping: Vec<(SourceType, DestinationType)>,
}

impl TypeMapping {
    /// Constructs a new [TypeMapping] from a list of [TypeParameter]s
    /// `type_parameters`. The [SourceType]s of the resulting [TypeMapping] are
    /// the [TypeId]s from `type_parameters` and the [DestinationType]s are the
    /// new [TypeId]s created from a transformation upon `type_parameters`.
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

    /// Constructs a new [TypeMapping] from a superset [TypeId] and a subset
    /// [TypeId]. The [SourceType]s of the resulting [TypeMapping] are the
    /// [TypeId]s from `superset` and the [DestinationType]s are the [TypeId]s
    /// from `subset`. Thus, the resulting [TypeMapping] maps the type
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
    /// So then the resulting [TypeMapping] would look like:
    ///
    /// ```ignore
    /// TypeMapping {
    ///     mapping: [
    ///         (L, u64),
    ///         (R, bool)
    ///     ]
    /// }
    /// ````
    ///
    /// So, as we can see, the resulting [TypeMapping] is a mapping from the
    /// type parameters of the `superset` to the type parameters of the
    /// `subset`. This [TypeMapping] can be used to complete monomorphization on
    /// methods, etc, that are implemented for the type of `superset` so that
    /// they can be used for `subset`.
    pub(crate) fn from_superset_and_subset(superset: TypeId, subset: TypeId) -> TypeMapping {
        match (look_up_type_id(superset), look_up_type_id(subset)) {
            (TypeInfo::UnknownGeneric { .. }, _) => TypeMapping {
                mapping: vec![(superset, subset)],
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

    /// Constructs a [TypeMapping] from a list of [TypeId]s `type_parameters`
    /// and a list of [TypeId]s `type_arguments`. The [SourceType]s of the
    /// resulting [TypeMapping] are the [TypeId]s from `type_parameters` and the
    /// [DestinationType]s are the [TypeId]s from `type_arguments`.
    fn from_type_parameters_and_type_arguments(
        type_parameters: Vec<SourceType>,
        type_arguments: Vec<DestinationType>,
    ) -> TypeMapping {
        let mapping = type_parameters
            .into_iter()
            .zip(type_arguments.into_iter())
            .collect::<Vec<_>>();
        TypeMapping { mapping }
    }

    /// Given a [TypeId] `type_id`, find (or create) a match for `type_id` in
    /// this [TypeMapping] and return it, if there is a match. Importantly, this
    /// function is recursive, so any `type_id` it's given will undergo
    /// recursive calls this function. For instance, in the case of
    /// [TypeInfo::Struct], both `fields` and `type_parameters` will recursively
    /// call `find_match` (via calling [CopyTypes]).
    ///
    /// A match can be found in two different circumstances:
    /// - `type_id` is a [TypeInfo::Custom] or [TypeInfo::UnknownGeneric]
    ///
    /// A match is created (i.e. a new `TypeId` is created) in these
    /// circumstances:
    /// - `type_id` is a [TypeInfo::Struct], [TypeInfo::Enum],
    ///     [TypeInfo::Array], or [TypeInfo::Tuple]
    /// - a new [TypeId] is created in these circumstances because `find_match`
    ///     descends recursively, and you can't be sure that it hasn't found a
    ///     match somewhere nested deeper in the type
    /// - TODO: there is a performance gain to be made here by having
    ///     `find_match` (or some `find_match_inner` return a `bool`). If that
    ///     `bool` is false, you know that there is no match found, and you can
    ///     be confident that even in the cases that otherwise would be creating
    ///     a new match, that no new match needs to be created, because there
    ///     were no nested matches
    ///
    /// A match cannot be found in any other circumstance
    ///
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
                let ary_ty_id = match self.find_match(ary_ty_id) {
                    Some(matching_id) => matching_id,
                    None => ary_ty_id,
                };
                Some(insert_type(TypeInfo::Array(
                    ary_ty_id,
                    count,
                    initial_elem_ty,
                )))
            }
            TypeInfo::Tuple(mut fields) => {
                fields.iter_mut().for_each(|field| field.copy_types(self));
                Some(insert_type(TypeInfo::Tuple(fields)))
            }
            TypeInfo::Unknown
            | TypeInfo::Str(..)
            | TypeInfo::UnsignedInteger(..)
            | TypeInfo::Boolean
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

    /// Unifies the given `type_arguments` with the [DestinationType]s of the
    /// [TypeMapping]
    pub(crate) fn unify_with_type_arguments(
        &self,
        type_arguments: &[TypeArgument],
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        for ((_, destination_type), type_arg) in self.mapping.iter().zip(type_arguments.iter()) {
            append!(
                unify(
                    *destination_type,
                    type_arg.type_id,
                    &type_arg.span,
                    "Type argument is not assignable to generic type parameter.",
                ),
                warnings,
                errors
            );
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
