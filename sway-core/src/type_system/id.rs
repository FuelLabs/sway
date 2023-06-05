use sway_error::error::CompileError;
use sway_types::{BaseIdent, Span};

use crate::{
    decl_engine::{DeclEngine, DeclEngineInsert},
    engine_threading::*,
    error::*,
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
        ctx: &mut CollectTypesMetadataContext,
    ) -> CompileResult<Vec<TypeMetadata>> {
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
        ok(res, vec![], vec![])
    }
}

impl ReplaceSelfType for TypeId {
    fn replace_self_type(&mut self, engines: &Engines, self_type: TypeId) {
        fn helper(type_id: TypeId, engines: &Engines, self_type: TypeId) -> Option<TypeId> {
            let type_engine = engines.te();
            let decl_engine = engines.de();
            match type_engine.get(type_id) {
                TypeInfo::TypeParam(_) => None,
                TypeInfo::SelfType => Some(self_type),
                TypeInfo::Enum(decl_ref) => {
                    let mut decl = decl_engine.get_enum(&decl_ref);
                    let mut need_to_create_new = false;

                    for variant in decl.variants.iter_mut() {
                        if let Some(type_id) =
                            helper(variant.type_argument.type_id, engines, self_type)
                        {
                            need_to_create_new = true;
                            variant.type_argument.type_id = type_id;
                        }
                    }

                    for type_param in decl.type_parameters.iter_mut() {
                        if let Some(type_id) = helper(type_param.type_id, engines, self_type) {
                            need_to_create_new = true;
                            type_param.type_id = type_id;
                        }
                    }

                    if need_to_create_new {
                        let new_decl_ref = decl_engine.insert(decl);
                        Some(type_engine.insert(engines, TypeInfo::Enum(new_decl_ref)))
                    } else {
                        None
                    }
                }
                TypeInfo::Struct(decl_ref) => {
                    let mut decl = decl_engine.get_struct(&decl_ref);
                    let mut need_to_create_new = false;

                    for field in decl.fields.iter_mut() {
                        if let Some(type_id) =
                            helper(field.type_argument.type_id, engines, self_type)
                        {
                            need_to_create_new = true;
                            field.type_argument.type_id = type_id;
                        }
                    }

                    for type_param in decl.type_parameters.iter_mut() {
                        if let Some(type_id) = helper(type_param.type_id, engines, self_type) {
                            need_to_create_new = true;
                            type_param.type_id = type_id;
                        }
                    }

                    if need_to_create_new {
                        let new_decl_ref = decl_engine.insert(decl);
                        Some(type_engine.insert(engines, TypeInfo::Struct(new_decl_ref)))
                    } else {
                        None
                    }
                }
                TypeInfo::Tuple(fields) => {
                    let mut need_to_create_new = false;
                    let fields = fields
                        .into_iter()
                        .map(|mut field| {
                            if let Some(type_id) = helper(field.type_id, engines, self_type) {
                                need_to_create_new = true;
                                field.type_id = type_id;
                            }
                            field
                        })
                        .collect::<Vec<_>>();
                    if need_to_create_new {
                        Some(type_engine.insert(engines, TypeInfo::Tuple(fields)))
                    } else {
                        None
                    }
                }
                TypeInfo::Custom {
                    call_path,
                    type_arguments,
                } => {
                    let mut need_to_create_new = false;
                    let type_arguments = type_arguments.map(|type_arguments| {
                        type_arguments
                            .into_iter()
                            .map(|mut type_arg| {
                                if let Some(type_id) = helper(type_arg.type_id, engines, self_type)
                                {
                                    need_to_create_new = true;
                                    type_arg.type_id = type_id;
                                }
                                type_arg
                            })
                            .collect::<Vec<_>>()
                    });
                    if need_to_create_new {
                        Some(type_engine.insert(
                            engines,
                            TypeInfo::Custom {
                                call_path,
                                type_arguments,
                            },
                        ))
                    } else {
                        None
                    }
                }
                TypeInfo::Array(mut elem_ty, count) => helper(elem_ty.type_id, engines, self_type)
                    .map(|type_id| {
                        elem_ty.type_id = type_id;
                        type_engine.insert(engines, TypeInfo::Array(elem_ty, count))
                    }),
                TypeInfo::Storage { fields } => {
                    let mut need_to_create_new = false;
                    let fields = fields
                        .into_iter()
                        .map(|mut field| {
                            if let Some(type_id) =
                                helper(field.type_argument.type_id, engines, self_type)
                            {
                                need_to_create_new = true;
                                field.type_argument.type_id = type_id;
                            }
                            field
                        })
                        .collect::<Vec<_>>();
                    if need_to_create_new {
                        Some(type_engine.insert(engines, TypeInfo::Storage { fields }))
                    } else {
                        None
                    }
                }
                TypeInfo::Alias { name, mut ty } => {
                    helper(ty.type_id, engines, self_type).map(|type_id| {
                        ty.type_id = type_id;
                        type_engine.insert(engines, TypeInfo::Alias { name, ty })
                    })
                }
                TypeInfo::Ptr(mut ty) => helper(ty.type_id, engines, self_type).map(|type_id| {
                    ty.type_id = type_id;
                    type_engine.insert(engines, TypeInfo::Ptr(ty))
                }),
                TypeInfo::Slice(mut ty) => helper(ty.type_id, engines, self_type).map(|type_id| {
                    ty.type_id = type_id;
                    type_engine.insert(engines, TypeInfo::Slice(ty))
                }),
                TypeInfo::Unknown
                | TypeInfo::UnknownGeneric { .. }
                | TypeInfo::Str(_)
                | TypeInfo::UnsignedInteger(_)
                | TypeInfo::Boolean
                | TypeInfo::ContractCaller { .. }
                | TypeInfo::B256
                | TypeInfo::Numeric
                | TypeInfo::RawUntypedPtr
                | TypeInfo::RawUntypedSlice
                | TypeInfo::Contract
                | TypeInfo::ErrorRecovery
                | TypeInfo::Placeholder(_) => None,
            }
        }

        if let Some(type_id) = helper(*self, engines, self_type) {
            *self = type_id;
        }
    }
}

impl SubstTypes for TypeId {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) {
        let type_engine = engines.te();
        if let Some(matching_id) = type_mapping.find_match(*self, engines) {
            if !matches!(type_engine.get(matching_id), TypeInfo::ErrorRecovery) {
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
        ctx: &TypeCheckContext,
        span: &Span,
        trait_constraints: Vec<TraitConstraint>,
    ) -> CompileResult<()> {
        let warnings = vec![];
        let mut errors = vec![];
        let engines = ctx.engines();

        let mut structure_generics = engines
            .te()
            .get(*self)
            .extract_inner_types_with_trait_constraints(engines);

        if !trait_constraints.is_empty() {
            structure_generics.insert(*self, trait_constraints);
        }

        for (structure_type_id, structure_trait_constraints) in &structure_generics {
            if structure_trait_constraints.is_empty() {
                continue;
            }

            let structure_type_info = engines.te().get(*structure_type_id);
            let structure_type_info_with_engines = engines.help_out(structure_type_info.clone());
            if let TypeInfo::UnknownGeneric {
                trait_constraints, ..
            } = &structure_type_info
            {
                let mut generic_trait_constraints_trait_names: Vec<CallPath<BaseIdent>> = vec![];
                for trait_constraint in trait_constraints.iter() {
                    generic_trait_constraints_trait_names.push(trait_constraint.trait_name.clone());
                }
                for structure_trait_constraint in structure_trait_constraints {
                    if !generic_trait_constraints_trait_names
                        .contains(&structure_trait_constraint.trait_name)
                    {
                        errors.push(CompileError::TraitConstraintMissing {
                            param: structure_type_info_with_engines.to_string(),
                            trait_name: structure_trait_constraint.trait_name.suffix.to_string(),
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
                        errors.push(CompileError::TraitConstraintNotSatisfied {
                            ty: structure_type_info_with_engines.to_string(),
                            trait_name: structure_trait_constraint.trait_name.suffix.to_string(),
                            span: span.clone(),
                        });
                    }
                }
            }
        }

        if errors.is_empty() {
            ok((), warnings, errors)
        } else {
            err(warnings, errors)
        }
    }
}
