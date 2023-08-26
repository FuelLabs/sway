use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{BaseIdent, Span};

use crate::{
    decl_engine::{DeclEngine, DeclEngineInsert},
    engine_threading::*,
    language::CallPath,
    semantic_analysis::TypeCheckContext,
    type_system::priv_prelude::*,
    types::*,
};

use std::{
    collections::{BTreeSet, HashMap},
    fmt,
};

/// A identifier to uniquely refer to our type terms
#[derive(PartialEq, Eq, Hash, Clone, Copy, Ord, PartialOrd, Debug)]
pub struct TypeId(usize);

impl DisplayWithEngines for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(f, "{}", engines.help_out(engines.te().get(*self)))
    }
}

impl DebugWithEngines for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(f, "{:?}", engines.help_out(engines.te().get(*self)))
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
        let possible = self.extract_any_including_self(engines, &filter_fn, vec![]);
        let mut res = vec![];
        for (type_id, _) in possible.into_iter() {
            match ctx.engines.te().get(type_id) {
                TypeInfo::UnknownGeneric { name, .. } => {
                    res.push(TypeMetadata::UnresolvedType(
                        name,
                        ctx.call_site_get(&type_id),
                    ));
                }
                TypeInfo::Placeholder(type_param) => {
                    res.push(TypeMetadata::UnresolvedType(
                        type_param.name_ident,
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
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) {
        let type_engine = engines.te();
        if let Some(matching_id) = type_mapping.find_match(*self, engines) {
            if !matches!(type_engine.get(matching_id), TypeInfo::ErrorRecovery(_)) {
                *self = matching_id;
            }
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
        let mut all_types: BTreeSet<TypeId> = type_engine.get(*self).extract_inner_types(engines);
        all_types.insert(*self);
        let type_parameter_info = type_engine.get(type_parameter.type_id);
        all_types
            .iter()
            .any(|type_id| type_engine.get(*type_id).eq(&type_parameter_info, engines))
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

    pub(crate) fn get_type_parameters(
        &self,
        type_engine: &TypeEngine,
        decl_engine: &DeclEngine,
    ) -> Option<Vec<TypeParameter>> {
        match type_engine.get(*self) {
            TypeInfo::Enum(decl_ref) => {
                let decl = decl_engine.get_enum(&decl_ref);
                (!decl.type_parameters.is_empty()).then_some(decl.type_parameters)
            }
            TypeInfo::Struct(decl_ref) => {
                let decl = decl_engine.get_struct(&decl_ref);
                (!decl.type_parameters.is_empty()).then_some(decl.type_parameters)
            }
            _ => None,
        }
    }

    /// Indicates of a given type is generic or not. Rely on whether the type is `Custom` and
    /// consider the special case where the resolved type is a struct or enum with a name that
    /// matches the name of the `Custom`.
    pub(crate) fn is_generic_parameter(
        self,
        type_engine: &TypeEngine,
        decl_engine: &DeclEngine,
        resolved_type_id: TypeId,
    ) -> bool {
        match (type_engine.get(self), type_engine.get(resolved_type_id)) {
            (TypeInfo::Custom { call_path, .. }, TypeInfo::Enum(decl_ref)) => {
                call_path.suffix != decl_engine.get_enum(&decl_ref).call_path.suffix
            }
            (TypeInfo::Custom { call_path, .. }, TypeInfo::Struct(decl_ref)) => {
                call_path.suffix != decl_engine.get_struct(&decl_ref).call_path.suffix
            }
            (TypeInfo::Custom { call_path, .. }, TypeInfo::Alias { name, .. }) => {
                call_path.suffix != name
            }
            (TypeInfo::Custom { .. }, _) => true,
            _ => false,
        }
    }

    pub(crate) fn extract_any_including_self<F>(
        &self,
        engines: &Engines,
        filter_fn: &F,
        trait_constraints: Vec<TraitConstraint>,
    ) -> HashMap<TypeId, Vec<TraitConstraint>>
    where
        F: Fn(&TypeInfo) -> bool,
    {
        let type_engine = engines.te();
        let type_info = type_engine.get(*self);
        let mut found = type_info.extract_any(engines, filter_fn);
        if filter_fn(&type_info) {
            found.insert(*self, trait_constraints);
        }
        found
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
        &self,
        handler: &Handler,
        ctx: &TypeCheckContext,
        span: &Span,
        trait_constraints: Vec<TraitConstraint>,
    ) -> Result<(), ErrorEmitted> {
        let engines = ctx.engines();

        let mut structure_generics = engines
            .te()
            .get(*self)
            .extract_inner_types_with_trait_constraints(engines);

        if !trait_constraints.is_empty() {
            structure_generics.insert(*self, trait_constraints);
        }

        handler.scope(|handler| {
            for (structure_type_id, structure_trait_constraints) in &structure_generics {
                if structure_trait_constraints.is_empty() {
                    continue;
                }

                // resolving trait constraits require a concrete type, we need to default numeric to u64
                engines
                    .te()
                    .decay_numeric(handler, engines, *structure_type_id, span)?;

                let structure_type_info = engines.te().get(*structure_type_id);
                let structure_type_info_with_engines =
                    engines.help_out(structure_type_info.clone());
                if let TypeInfo::UnknownGeneric {
                    trait_constraints, ..
                } = &structure_type_info
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
                    let generic_trait_constraints_trait_names = ctx
                        .namespace
                        .implemented_traits
                        .get_trait_names_for_type(engines, *structure_type_id);
                    for structure_trait_constraint in structure_trait_constraints {
                        if !generic_trait_constraints_trait_names.contains(
                            &structure_trait_constraint
                                .trait_name
                                .to_fullpath(ctx.namespace),
                        ) {
                            handler.emit_err(CompileError::TraitConstraintNotSatisfied {
                                ty: structure_type_info_with_engines.to_string(),
                                trait_name: structure_trait_constraint
                                    .trait_name
                                    .suffix
                                    .to_string(),
                                span: span.clone(),
                            });
                        }
                    }
                }
            }
            Ok(())
        })
    }
}
