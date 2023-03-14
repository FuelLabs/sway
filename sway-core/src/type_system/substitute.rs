use itertools::Itertools;
use std::{
    collections::BTreeMap,
    fmt,
    hash::Hasher,
    slice::{Iter, IterMut},
    vec::IntoIter,
};
use sway_error::error::CompileError;
use sway_types::{Ident, Span, Spanned};

use super::*;
use crate::{decl_engine::DeclEngine, engine_threading::*, namespace::Path, Namespace};

pub trait SubstTypes {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>);

    fn subst(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        if !type_mapping.is_empty() {
            self.subst_inner(type_mapping, engines);
        }
    }
}

/// A list of types that serve as the list of type params for type substitution.
/// Any types of the [TypeParam][TypeInfo::TypeParam] variant will point to an index in
/// this list.
#[derive(Debug, Clone, Default)]
pub struct TypeSubstList {
    self_type: Option<TypeParameter>,
    type_params: BTreeMap<String, TypeParameter>,
}

impl TypeSubstList {
    pub(crate) fn new() -> TypeSubstList {
        TypeSubstList {
            self_type: None,
            type_params: BTreeMap::new(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        todo!();
        // self.type_params.is_empty()
    }

    pub(crate) fn len(&self) -> usize {
        todo!();
        // self.type_params.len()
    }

    pub(crate) fn push(&mut self, type_param: TypeParameter) {
        todo!();
        // self.type_params.push(type_param);
    }

    pub(crate) fn iter(&self) -> Iter<'_, TypeParameter> {
        todo!();
        // self.type_params.iter()
    }

    pub(crate) fn into_iter(self) -> IntoIter<TypeParameter> {
        todo!();
        // self.type_params.into_iter()
    }

    pub(crate) fn iter_mut(&mut self) -> IterMut<'_, TypeParameter> {
        todo!();
        // self.type_params.iter_mut()
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn monomorphize(
        &mut self,
        namespace: &mut Namespace,
        engines: Engines<'_>,
        mod_path: &Path,
        type_arguments: &mut [TypeArgument],
        enforce_type_arguments: EnforceTypeArguments,
        obj_name: &Ident,
        call_site_span: &Span,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let type_engine = engines.te();
        let decl_engine = engines.de();

        // Create the type mapping.
        // Exit early if needed/possible.
        let type_mapping = match (self.len(), type_arguments.len()) {
            (0, 0) => {
                // Exit early if there are no types to substitute and if there
                // are no type arguments.
                return ok((), warnings, errors);
            }
            (n, 0) if n > 0 && enforce_type_arguments.is_yes() => {
                // Exit early if there are types to substitute but no type
                // arguments were provided.
                errors.push(CompileError::NeedsTypeArguments {
                    name: obj_name.clone(),
                    span: call_site_span.clone(),
                });
                return err(warnings, errors);
            }
            (0, m) if m > 0 => {
                // Exit early if there are no types to substitute but type
                // arguments were provided.
                let type_arguments_span = type_arguments
                    .iter()
                    .map(|x| x.span.clone())
                    .reduce(Span::join)
                    .unwrap_or_else(|| obj_name.span());
                errors.push(CompileError::DoesNotTakeTypeArguments {
                    name: obj_name.clone(),
                    span: type_arguments_span,
                });
                return err(warnings, errors);
            }
            (n, m) if n != m => {
                // Exit early if the number of types to substitute and the
                // number of type arguments differ.
                let type_arguments_span = type_arguments
                    .iter()
                    .map(|x| x.span.clone())
                    .reduce(Span::join)
                    .unwrap_or_else(|| obj_name.span());
                errors.push(CompileError::IncorrectNumberOfTypeArguments {
                    given: type_arguments.len(),
                    expected: self.len(),
                    span: type_arguments_span,
                });
                return err(warnings, errors);
            }
            (n, m) if n == m => {
                for type_argument in type_arguments.iter_mut() {
                    type_argument.type_id = check!(
                        type_engine.resolve(
                            decl_engine,
                            type_argument.type_id,
                            &type_argument.span,
                            enforce_type_arguments,
                            None,
                            namespace,
                            mod_path
                        ),
                        type_engine.insert(decl_engine, TypeInfo::ErrorRecovery),
                        warnings,
                        errors
                    );
                }
                TypeSubstMap::from_type_parameters_and_type_arguments(
                    self.iter().map(|type_param| type_param.type_id).collect(),
                    type_arguments
                        .iter()
                        .map(|type_arg| type_arg.type_id)
                        .collect(),
                )
            }
            (_, _) => {
                // The only case left is:
                // (n, 0) if n > 0 && enforce_type_arguments.is_no()
                TypeSubstMap::from_type_parameters(engines, self.iter())
            }
        };

        self.subst(&type_mapping, engines);
        ok((), warnings, errors)
    }

    // /// Given a `value` of type `T` that is able to be monomorphized and a set
    // /// of `type_arguments`, monomorphize `value` with the `type_arguments`.
    // ///
    // /// When this function is called, it is passed a `T` that is a copy of some
    // /// original declaration for `T` (let's denote the original with `[T]`).
    // /// Because monomorphization happens at application time (e.g. function
    // /// application), we want to be able to modify `value` such that type
    // /// checking the application of `value` affects only `T` and not `[T]`.
    // ///
    // /// So, at a high level, this function does two things. It 1) performs the
    // /// necessary work to refresh the relevant generic types in `T` so that they
    // /// are distinct from the generics of the same name in `[T]`. And it 2)
    // /// applies `type_arguments` (if any are provided) to the type parameters
    // /// of `value`, unifying the types.
    // ///
    // /// There are 4 cases that are handled in this function:
    // ///
    // /// 1. `value` does not have type parameters + `type_arguments` is empty:
    // ///     1a. return ok
    // /// 2. `value` has type parameters + `type_arguments` is empty:
    // ///     2a. if the [EnforceTypeArguments::Yes] variant is provided, then
    // ///         error
    // ///     2b. refresh the generic types with a [TypeSubstMapping]
    // /// 3. `value` does have type parameters + `type_arguments` is nonempty:
    // ///     3a. error
    // /// 4. `value` has type parameters + `type_arguments` is nonempty:
    // ///     4a. check to see that the type parameters and `type_arguments` have
    // ///         the same length
    // ///     4b. for each type argument in `type_arguments`, resolve the type
    // ///     4c. refresh the generic types with a [TypeSubstMapping]
    // #[allow(clippy::too_many_arguments)]
    // pub(crate) fn monomorphize<T>(
    //     &self,
    //     decl_engine: &DeclEngine,
    //     value: &mut T,
    //     type_arguments: &mut [TypeArgument],
    //     enforce_type_arguments: EnforceTypeArguments,
    //     call_site_span: &Span,
    //     namespace: &mut Namespace,
    //     mod_path: &Path,
    // ) -> CompileResult<()>
    // where
    //     T: MonomorphizeHelper + SubstTypes,
    // {
    //     let mut warnings = vec![];
    //     let mut errors = vec![];
    //     let engines = Engines::new(self, decl_engine);
    //     match (
    //         value.type_parameters().is_empty(),
    //         type_arguments.is_empty(),
    //     ) {
    //         (true, true) => ok((), warnings, errors),
    //         (false, true) => {
    //             if let EnforceTypeArguments::Yes = enforce_type_arguments {
    //                 errors.push(CompileError::NeedsTypeArguments {
    //                     name: value.name().clone(),
    //                     span: call_site_span.clone(),
    //                 });
    //                 return err(warnings, errors);
    //             }
    //             let type_mapping =
    //                 TypeSubstMap::from_type_parameters(engines, value.type_parameters());
    //             value.subst(&type_mapping, engines);
    //             ok((), warnings, errors)
    //         }
    //         (true, false) => {
    //             let type_arguments_span = type_arguments
    //                 .iter()
    //                 .map(|x| x.span.clone())
    //                 .reduce(Span::join)
    //                 .unwrap_or_else(|| value.name().span());
    //             errors.push(CompileError::DoesNotTakeTypeArguments {
    //                 name: value.name().clone(),
    //                 span: type_arguments_span,
    //             });
    //             err(warnings, errors)
    //         }
    //         (false, false) => {
    //             let type_arguments_span = type_arguments
    //                 .iter()
    //                 .map(|x| x.span.clone())
    //                 .reduce(Span::join)
    //                 .unwrap_or_else(|| value.name().span());
    //             if value.type_parameters().len() != type_arguments.len() {
    //                 errors.push(CompileError::IncorrectNumberOfTypeArguments {
    //                     given: type_arguments.len(),
    //                     expected: value.type_parameters().len(),
    //                     span: type_arguments_span,
    //                 });
    //                 return err(warnings, errors);
    //             }
    //             for type_argument in type_arguments.iter_mut() {
    //                 type_argument.type_id = check!(
    //                     self.resolve(
    //                         decl_engine,
    //                         type_argument.type_id,
    //                         &type_argument.span,
    //                         enforce_type_arguments,
    //                         None,
    //                         namespace,
    //                         mod_path
    //                     ),
    //                     self.insert(decl_engine, TypeInfo::ErrorRecovery),
    //                     warnings,
    //                     errors
    //                 );
    //             }
    //             let type_mapping = TypeSubstMap::from_type_parameters_and_type_arguments(
    //                 value
    //                     .type_parameters()
    //                     .iter()
    //                     .map(|type_param| type_param.type_id)
    //                     .collect(),
    //                 type_arguments
    //                     .iter()
    //                     .map(|type_arg| type_arg.type_id)
    //                     .collect(),
    //             );
    //             value.subst(&type_mapping, engines);
    //             ok((), warnings, errors)
    //         }
    //     }
    // }
}

impl EqWithEngines for TypeSubstList {}
impl PartialEqWithEngines for TypeSubstList {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let TypeSubstList {
            self_type: lst,
            type_params: ll,
        } = self;
        let TypeSubstList {
            self_type: rst,
            type_params: rl,
        } = other;
        lst.eq(rst, engines)
            && ll
                .values()
                .collect_vec()
                .eq(&rl.values().collect_vec(), engines)
    }
}

impl HashWithEngines for TypeSubstList {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TypeSubstList {
            self_type,
            type_params,
        } = self;
        self_type.hash(state, engines);
        type_params.values().collect_vec().hash(state, engines);
    }
}

impl OrdWithEngines for TypeSubstList {
    fn cmp(&self, other: &Self, engines: Engines<'_>) -> std::cmp::Ordering {
        let TypeSubstList {
            self_type: lst,
            type_params: ll,
        } = self;
        let TypeSubstList {
            self_type: rst,
            type_params: rl,
        } = other;
        lst.cmp(rst, engines).then_with(|| {
            ll.values()
                .collect_vec()
                .cmp(&rl.values().collect_vec(), engines)
        })
    }
}

impl SubstTypes for TypeSubstList {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        if let Some(ref mut self_type) = self.self_type {
            self_type.subst(type_mapping, engines);
        }
        self.type_params
            .values_mut()
            .for_each(|x| x.subst(type_mapping, engines));
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
    pub(crate) fn from_type_parameters<'a, T>(
        engines: Engines<'_>,
        type_parameters: T,
    ) -> TypeSubstMap
    where
        T: Iterator<Item = &'a TypeParameter>,
    {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        let mapping = type_parameters
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
        decl_engine: &DeclEngine,
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
                    decl_engine,
                    type_parameters,
                    type_arguments,
                )
            }
            (TypeInfo::Enum(decl_ref_params), TypeInfo::Enum(decl_ref_args)) => {
                let decl_params = decl_engine.get_enum(&decl_ref_params);
                let decl_args = decl_engine.get_enum(&decl_ref_args);
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
                let decl_params = decl_engine.get_struct(&decl_ref_params);
                let decl_args = decl_engine.get_struct(&decl_ref_args);

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
            TypeInfo::TypeParam { .. } => None,
            TypeInfo::Struct(decl_ref) => {
                let mut need_to_create_new = false;
                let mut decl_ref = decl_ref.clone();
                for type_param in decl_ref.subst_list_mut().iter_mut() {
                    if let Some(type_id) = self.find_match(type_param.type_id, engines) {
                        need_to_create_new = true;
                        type_param.type_id = type_id;
                    }
                }
                if need_to_create_new {
                    Some(type_engine.insert(decl_engine, TypeInfo::Struct(decl_ref)))
                } else {
                    None
                }
            }
            TypeInfo::Enum(decl_ref) => {
                let mut need_to_create_new = false;
                let mut decl_ref = decl_ref.clone();
                for type_param in decl_ref.subst_list_mut().iter_mut() {
                    if let Some(type_id) = self.find_match(type_param.type_id, engines) {
                        need_to_create_new = true;
                        type_param.type_id = type_id;
                    }
                }
                if need_to_create_new {
                    Some(type_engine.insert(decl_engine, TypeInfo::Enum(decl_ref)))
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
