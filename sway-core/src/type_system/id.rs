use crate::{
    decl_engine::{DeclEngine, DeclEngineInsert},
    engine_threading::*,
    error::*,
    type_system::priv_prelude::*,
    types::*,
};

use std::{collections::BTreeSet, fmt};

/// A identifier to uniquely refer to our type terms
#[derive(PartialEq, Eq, Hash, Clone, Copy, Ord, PartialOrd, Debug)]
pub struct TypeId(usize);

impl DisplayWithEngines for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: Engines<'_>) -> fmt::Result {
        write!(f, "{}", engines.help_out(engines.te().get(*self)))
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
        let engines = Engines::new(ctx.type_engine, ctx.decl_engine);
        let possible = self.extract_any_including_self(engines, &filter_fn);
        let mut res = vec![];
        for type_id in possible.into_iter() {
            match ctx.type_engine.get(type_id) {
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
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        fn helper(type_id: TypeId, engines: Engines<'_>, self_type: TypeId) -> Option<TypeId> {
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
                        Some(type_engine.insert(decl_engine, TypeInfo::Enum(new_decl_ref)))
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
                        Some(type_engine.insert(decl_engine, TypeInfo::Struct(new_decl_ref)))
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
                        Some(type_engine.insert(decl_engine, TypeInfo::Tuple(fields)))
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
                            decl_engine,
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
                        type_engine.insert(decl_engine, TypeInfo::Array(elem_ty, count))
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
                        Some(type_engine.insert(decl_engine, TypeInfo::Storage { fields }))
                    } else {
                        None
                    }
                }
                TypeInfo::Alias { name, mut ty } => {
                    helper(ty.type_id, engines, self_type).map(|type_id| {
                        ty.type_id = type_id;
                        type_engine.insert(decl_engine, TypeInfo::Alias { name, ty })
                    })
                }
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
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
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
        engines: Engines<'_>,
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
        engines: Engines<'_>,
        filter_fn: &F,
    ) -> BTreeSet<TypeId>
    where
        F: Fn(&TypeInfo) -> bool,
    {
        let type_engine = engines.te();
        let type_info = type_engine.get(*self);
        let mut found: BTreeSet<TypeId> = type_info.extract_any(engines, filter_fn);
        if filter_fn(&type_info) {
            found.insert(*self);
        }
        found
    }
}
