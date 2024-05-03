use indexmap::IndexMap;
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{BaseIdent, Span};

use crate::{
    engine_threading::{
        DebugWithEngines, DisplayWithEngines, Engines, PartialEqWithEngines,
        PartialEqWithEnginesContext, WithEngines,
    },
    language::CallPath,
    semantic_analysis::type_check_context::EnforceTypeArguments,
    semantic_analysis::TypeCheckContext,
    type_system::priv_prelude::*,
    types::{
        CollectTypesMetadata, CollectTypesMetadataContext, TypeMetadata,
        UnconstrainedTypeParameters,
    },
};

use std::{
    collections::{BTreeSet, HashSet},
    fmt,
};

const EXTRACT_ANY_MAX_DEPTH: usize = 128;

/// A identifier to uniquely refer to our type terms
#[derive(PartialEq, Eq, Hash, Clone, Copy, Ord, PartialOrd, Debug)]
pub struct TypeId(usize);

impl DisplayWithEngines for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(f, "{}", engines.help_out(&*engines.te().get(*self)))
    }
}

impl DebugWithEngines for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(f, "{:?}", engines.help_out(&*engines.te().get(*self)))
    }
}

impl From<usize> for TypeId {
    fn from(o: usize) -> Self {
        TypeId(o)
    }
}

impl CollectTypesMetadata for TypeId {
    fn collect_types_metadata(
        &self,
        _handler: &Handler,
        ctx: &mut CollectTypesMetadataContext,
    ) -> Result<Vec<TypeMetadata>, ErrorEmitted> {
        fn filter_fn(type_info: &TypeInfo) -> bool {
            matches!(type_info, TypeInfo::UnknownGeneric { .. })
                || matches!(type_info, TypeInfo::Placeholder(_))
        }
        let engines = ctx.engines;
        let possible = self.extract_any_including_self(engines, &filter_fn, vec![], 0);
        let mut res = vec![];
        for (type_id, _) in possible {
            match &*ctx.engines.te().get(type_id) {
                TypeInfo::UnknownGeneric { name, .. } => {
                    res.push(TypeMetadata::UnresolvedType(
                        name.clone(),
                        ctx.call_site_get(&type_id),
                    ));
                }
                TypeInfo::Placeholder(type_param) => {
                    res.push(TypeMetadata::UnresolvedType(
                        type_param.name_ident.clone(),
                        ctx.call_site_get(self),
                    ));
                }
                _ => {}
            }
        }
        Ok(res)
    }
}

impl SubstTypes for TypeId {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> HasChanges {
        let type_engine = engines.te();
        if let Some(matching_id) = type_mapping.find_match(*self, engines) {
            if !matches!(&*type_engine.get(matching_id), TypeInfo::ErrorRecovery(_)) {
                *self = matching_id;
                HasChanges::Yes
            } else {
                HasChanges::No
            }
        } else {
            HasChanges::No
        }
    }
}

impl UnconstrainedTypeParameters for TypeId {
    fn type_parameter_is_unconstrained(
        &self,
        engines: &Engines,
        type_parameter: &TypeParameter,
    ) -> bool {
        let type_engine = engines.te();
        let mut all_types: BTreeSet<TypeId> = self.extract_inner_types(engines);
        all_types.insert(*self);
        let type_parameter_info = type_engine.get(type_parameter.type_id);
        all_types.iter().any(|type_id| {
            type_engine.get(*type_id).eq(
                &type_parameter_info,
                &PartialEqWithEnginesContext::new(engines),
            )
        })
    }
}

impl TypeId {
    pub(super) fn new(index: usize) -> TypeId {
        TypeId(index)
    }

    /// Returns the index that identifies the type.
    pub fn index(&self) -> usize {
        self.0
    }

    pub(crate) fn get_type_parameters(self, engines: &Engines) -> Option<Vec<TypeParameter>> {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        match &*type_engine.get(self) {
            TypeInfo::Enum(decl_ref) => {
                let decl = decl_engine.get_enum(decl_ref);
                (!decl.type_parameters.is_empty()).then_some(decl.type_parameters.clone())
            }
            TypeInfo::Struct(decl_ref) => {
                let decl = decl_engine.get_struct(decl_ref);
                (!decl.type_parameters.is_empty()).then_some(decl.type_parameters.clone())
            }
            _ => None,
        }
    }

    /// Indicates of a given type is generic or not. Rely on whether the type is `Custom` and
    /// consider the special case where the resolved type is a struct or enum with a name that
    /// matches the name of the `Custom`.
    pub(crate) fn is_generic_parameter(self, engines: &Engines, resolved_type_id: TypeId) -> bool {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        match (&*type_engine.get(self), &*type_engine.get(resolved_type_id)) {
            (
                TypeInfo::Custom {
                    qualified_call_path: call_path,
                    ..
                },
                TypeInfo::Enum(decl_ref),
            ) => call_path.call_path.suffix != decl_engine.get_enum(decl_ref).call_path.suffix,
            (
                TypeInfo::Custom {
                    qualified_call_path: call_path,
                    ..
                },
                TypeInfo::Struct(decl_ref),
            ) => call_path.call_path.suffix != decl_engine.get_struct(decl_ref).call_path.suffix,
            (
                TypeInfo::Custom {
                    qualified_call_path: call_path,
                    ..
                },
                TypeInfo::Alias { name, .. },
            ) => call_path.call_path.suffix != name.clone(),
            (TypeInfo::Custom { .. }, _) => true,
            _ => false,
        }
    }

    pub(crate) fn extract_any_including_self<F>(
        self,
        engines: &Engines,
        filter_fn: &F,
        trait_constraints: Vec<TraitConstraint>,
        depth: usize,
    ) -> IndexMap<TypeId, Vec<TraitConstraint>>
    where
        F: Fn(&TypeInfo) -> bool,
    {
        let type_engine = engines.te();
        let type_info = type_engine.get(self);
        let mut found = self.extract_any(engines, filter_fn, depth + 1);
        if filter_fn(&type_info) {
            found.insert(self, trait_constraints);
        }
        found
    }

    pub(crate) fn extract_any<F>(
        self,
        engines: &Engines,
        filter_fn: &F,
        depth: usize,
    ) -> IndexMap<TypeId, Vec<TraitConstraint>>
    where
        F: Fn(&TypeInfo) -> bool,
    {
        if depth >= EXTRACT_ANY_MAX_DEPTH {
            panic!("Possible infinite recursion at extract_any");
        }

        fn extend(
            hashmap: &mut IndexMap<TypeId, Vec<TraitConstraint>>,
            hashmap_other: IndexMap<TypeId, Vec<TraitConstraint>>,
        ) {
            for (type_id, trait_constraints) in hashmap_other {
                if let Some(existing_trait_constraints) = hashmap.get_mut(&type_id) {
                    existing_trait_constraints.extend(trait_constraints);
                } else {
                    hashmap.insert(type_id, trait_constraints);
                }
            }
        }

        let decl_engine = engines.de();
        let mut found: IndexMap<TypeId, Vec<TraitConstraint>> = IndexMap::new();
        match &*engines.te().get(self) {
            TypeInfo::Unknown
            | TypeInfo::Never
            | TypeInfo::Placeholder(_)
            | TypeInfo::TypeParam(_)
            | TypeInfo::StringArray(_)
            | TypeInfo::StringSlice
            | TypeInfo::UnsignedInteger(_)
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::Boolean
            | TypeInfo::B256
            | TypeInfo::Numeric
            | TypeInfo::Contract
            | TypeInfo::ErrorRecovery(_)
            | TypeInfo::TraitType { .. } => {}
            TypeInfo::Enum(enum_ref) => {
                let enum_decl = decl_engine.get_enum(enum_ref);
                for type_param in &enum_decl.type_parameters {
                    extend(
                        &mut found,
                        type_param.type_id.extract_any_including_self(
                            engines,
                            filter_fn,
                            type_param.trait_constraints.clone(),
                            depth + 1,
                        ),
                    );
                }
                for variant in &enum_decl.variants {
                    extend(
                        &mut found,
                        variant.type_argument.type_id.extract_any_including_self(
                            engines,
                            filter_fn,
                            vec![],
                            depth + 1,
                        ),
                    );
                }
            }
            TypeInfo::Struct(struct_ref) => {
                let struct_decl = decl_engine.get_struct(struct_ref);
                for type_param in &struct_decl.type_parameters {
                    extend(
                        &mut found,
                        type_param.type_id.extract_any_including_self(
                            engines,
                            filter_fn,
                            type_param.trait_constraints.clone(),
                            depth + 1,
                        ),
                    );
                }
                for field in &struct_decl.fields {
                    extend(
                        &mut found,
                        field.type_argument.type_id.extract_any_including_self(
                            engines,
                            filter_fn,
                            vec![],
                            depth + 1,
                        ),
                    );
                }
            }
            TypeInfo::Tuple(elems) => {
                for elem in elems {
                    extend(
                        &mut found,
                        elem.type_id.extract_any_including_self(
                            engines,
                            filter_fn,
                            vec![],
                            depth + 1,
                        ),
                    );
                }
            }
            TypeInfo::ContractCaller {
                abi_name: _,
                address,
            } => {
                if let Some(address) = address {
                    extend(
                        &mut found,
                        address.return_type.extract_any_including_self(
                            engines,
                            filter_fn,
                            vec![],
                            depth + 1,
                        ),
                    );
                }
            }
            TypeInfo::Custom {
                qualified_call_path: _,
                type_arguments,
                root_type_id: _,
            } => {
                if let Some(type_arguments) = type_arguments {
                    for type_arg in type_arguments {
                        extend(
                            &mut found,
                            type_arg.type_id.extract_any_including_self(
                                engines,
                                filter_fn,
                                vec![],
                                depth + 1,
                            ),
                        );
                    }
                }
            }
            TypeInfo::Array(ty, _) => {
                extend(
                    &mut found,
                    ty.type_id
                        .extract_any_including_self(engines, filter_fn, vec![], depth + 1),
                );
            }
            TypeInfo::Storage { fields } => {
                for field in fields {
                    extend(
                        &mut found,
                        field.type_argument.type_id.extract_any_including_self(
                            engines,
                            filter_fn,
                            vec![],
                            depth + 1,
                        ),
                    );
                }
            }
            TypeInfo::Alias { name: _, ty } => {
                extend(
                    &mut found,
                    ty.type_id
                        .extract_any_including_self(engines, filter_fn, vec![], depth + 1),
                );
            }
            TypeInfo::UnknownGeneric {
                name: _,
                trait_constraints,
                parent: _,
                is_from_type_parameter: _,
            } => {
                found.insert(self, trait_constraints.to_vec());
                for trait_constraint in trait_constraints.iter() {
                    for type_arg in &trait_constraint.type_arguments {
                        // In case type_id was already added skip it.
                        // This is required because of recursive generic trait such as `T: Trait<T>`
                        if !found.contains_key(&type_arg.type_id) {
                            extend(
                                &mut found,
                                type_arg.type_id.extract_any_including_self(
                                    engines,
                                    filter_fn,
                                    vec![],
                                    depth + 1,
                                ),
                            );
                        }
                    }
                }
            }
            TypeInfo::Ptr(ty) => {
                extend(
                    &mut found,
                    ty.type_id
                        .extract_any_including_self(engines, filter_fn, vec![], depth + 1),
                );
            }
            TypeInfo::Slice(ty) => {
                extend(
                    &mut found,
                    ty.type_id
                        .extract_any_including_self(engines, filter_fn, vec![], depth + 1),
                );
            }
            TypeInfo::Ref {
                referenced_type, ..
            } => {
                extend(
                    &mut found,
                    referenced_type.type_id.extract_any_including_self(
                        engines,
                        filter_fn,
                        vec![],
                        depth + 1,
                    ),
                );
            }
        }
        found
    }

    /// Given a `TypeId` `self`, analyze `self` and return all inner
    /// `TypeId`'s of `self`, not including `self`.
    pub(crate) fn extract_inner_types(&self, engines: &Engines) -> BTreeSet<TypeId> {
        fn filter_fn(_type_info: &TypeInfo) -> bool {
            true
        }
        self.extract_any(engines, &filter_fn, 0)
            .keys()
            .copied()
            .collect()
    }

    pub(crate) fn extract_inner_types_with_trait_constraints(
        self,
        engines: &Engines,
    ) -> IndexMap<TypeId, Vec<TraitConstraint>> {
        fn filter_fn(_type_info: &TypeInfo) -> bool {
            true
        }
        self.extract_any(engines, &filter_fn, 0)
    }

    /// Given a `TypeId` `self`, analyze `self` and return all nested
    /// `TypeInfo`'s found in `self`, including `self`.
    pub(crate) fn extract_nested_types(self, engines: &Engines) -> Vec<TypeInfo> {
        let type_engine = engines.te();
        let mut inner_types: Vec<TypeInfo> = self
            .extract_inner_types(engines)
            .into_iter()
            .map(|type_id| (*type_engine.get(type_id)).clone())
            .collect();
        inner_types.push((*type_engine.get(self)).clone());
        inner_types
    }

    pub(crate) fn extract_nested_generics(
        self,
        engines: &Engines,
    ) -> HashSet<WithEngines<'_, TypeInfo>> {
        let nested_types = self.extract_nested_types(engines);
        HashSet::from_iter(nested_types.into_iter().filter_map(|x| match x {
            TypeInfo::UnknownGeneric { .. } => Some(WithEngines::new(x, engines)),
            _ => None,
        }))
    }

    /// `check_type_parameter_bounds` does two types of checks. Lets use the example below for demonstrating the two checks:
    /// ```ignore
    /// enum MyEnum<T> where T: MyAdd {
    ///   X: T,
    /// }
    /// ```
    /// The enum above has a constraint where `T` should implement the trait `MyAdd`.
    ///
    /// If `check_type_parameter_bounds` is called on type `MyEnum<u64>` and `u64`
    /// does not implement the trait `MyAdd` then the error `CompileError::TraitConstraintNotSatisfied`
    /// is thrown.
    ///
    /// The second type of check performed results in an error for the example below.
    /// ```ignore
    /// fn add2<G>(e: MyEnum<G>) -> G {
    /// }
    /// ```
    /// If `check_type_parameter_bounds` is called on type `MyEnum<G>` and the type parameter `G`
    /// does not have the trait constraint `where G: MyAdd` then the error `CompileError::TraitConstraintMissing`
    /// is thrown.
    pub(crate) fn check_type_parameter_bounds(
        self,
        handler: &Handler,
        mut ctx: TypeCheckContext,
        span: &Span,
        type_param: Option<TypeParameter>,
    ) -> Result<(), ErrorEmitted> {
        let engines = ctx.engines();

        let mut structure_generics = self.extract_inner_types_with_trait_constraints(engines);

        if let Some(type_param) = type_param {
            if !type_param.trait_constraints.is_empty() {
                structure_generics.insert(self, type_param.trait_constraints);
            }
        }

        handler.scope(|handler| {
            for (structure_type_id, structure_trait_constraints) in &structure_generics {
                if structure_trait_constraints.is_empty() {
                    continue;
                }

                // resolving trait constraints require a concrete type, we need to default numeric to u64
                engines
                    .te()
                    .decay_numeric(handler, engines, *structure_type_id, span)?;

                let structure_type_info = engines.te().get(*structure_type_id);
                let structure_type_info_with_engines = engines.help_out(&*structure_type_info);
                if let TypeInfo::UnknownGeneric {
                    trait_constraints, ..
                } = &*structure_type_info
                {
                    let mut generic_trait_constraints_trait_names: Vec<CallPath<BaseIdent>> =
                        vec![];
                    for trait_constraint in trait_constraints.iter() {
                        generic_trait_constraints_trait_names
                            .push(trait_constraint.trait_name.clone());
                    }
                    for structure_trait_constraint in structure_trait_constraints {
                        if !generic_trait_constraints_trait_names
                            .contains(&structure_trait_constraint.trait_name)
                        {
                            handler.emit_err(CompileError::TraitConstraintMissing {
                                param: structure_type_info_with_engines.to_string(),
                                trait_name: structure_trait_constraint
                                    .trait_name
                                    .suffix
                                    .to_string(),
                                span: span.clone(),
                            });
                        }
                    }
                } else {
                    let found_error = self.check_trait_constraints_errors(
                        handler,
                        ctx.by_ref(),
                        structure_type_id,
                        structure_trait_constraints,
                        |_| {},
                    );
                    if found_error {
                        // Retrieve the implemented traits for the type and insert them in the namespace.
                        // insert_trait_implementation_for_type is done lazily only when required because of a failure.
                        ctx.insert_trait_implementation_for_type(*structure_type_id);
                        self.check_trait_constraints_errors(
                            handler,
                            ctx.by_ref(),
                            structure_type_id,
                            structure_trait_constraints,
                            |structure_trait_constraint| {
                                let mut type_arguments_string = String::new();
                                if !structure_trait_constraint.type_arguments.is_empty() {
                                    type_arguments_string = format!(
                                        "<{}>",
                                        engines.help_out(
                                            structure_trait_constraint.type_arguments.clone()
                                        )
                                    );
                                }
                                handler.emit_err(CompileError::TraitConstraintNotSatisfied {
                                    type_id: structure_type_id.index(),
                                    ty: structure_type_info_with_engines.to_string(),
                                    trait_name: format!(
                                        "{}{}",
                                        structure_trait_constraint.trait_name.suffix,
                                        type_arguments_string
                                    ),
                                    span: span.clone(),
                                });
                            },
                        );
                    }
                }
            }
            Ok(())
        })
    }

    fn check_trait_constraints_errors(
        self,
        handler: &Handler,
        mut ctx: TypeCheckContext,
        structure_type_id: &TypeId,
        structure_trait_constraints: &Vec<TraitConstraint>,
        f: impl Fn(&TraitConstraint),
    ) -> bool {
        let engines = ctx.engines();
        let unify_check = UnifyCheck::non_dynamic_equality(engines);
        let mut found_error = false;
        let generic_trait_constraints_trait_names_and_args = ctx
            .namespace()
            .module(ctx.engines())
            .current_items()
            .implemented_traits
            .get_trait_names_and_type_arguments_for_type(engines, *structure_type_id);
        for structure_trait_constraint in structure_trait_constraints {
            let structure_trait_constraint_trait_name = &structure_trait_constraint
                .trait_name
                .to_fullpath(ctx.engines(), ctx.namespace());
            if !generic_trait_constraints_trait_names_and_args.iter().any(
                |(trait_name, trait_args)| {
                    trait_name == structure_trait_constraint_trait_name
                        && trait_args.len() == structure_trait_constraint.type_arguments.len()
                        && trait_args
                            .iter()
                            .zip(structure_trait_constraint.type_arguments.iter())
                            .all(|(t1, t2)| {
                                unify_check.check(
                                    ctx.resolve_type(
                                        handler,
                                        t1.type_id,
                                        &t1.span,
                                        EnforceTypeArguments::No,
                                        None,
                                    )
                                    .unwrap_or_else(|err| {
                                        engines.te().insert(
                                            engines,
                                            TypeInfo::ErrorRecovery(err),
                                            None,
                                        )
                                    }),
                                    ctx.resolve_type(
                                        handler,
                                        t2.type_id,
                                        &t2.span,
                                        EnforceTypeArguments::No,
                                        None,
                                    )
                                    .unwrap_or_else(|err| {
                                        engines.te().insert(
                                            engines,
                                            TypeInfo::ErrorRecovery(err),
                                            None,
                                        )
                                    }),
                                )
                            })
                },
            ) {
                found_error = true;
                f(structure_trait_constraint);
            }
        }
        found_error
    }
}
