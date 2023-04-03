use crate::{
    decl_engine::DeclEngine, engine_threading::*, error::*, type_system::priv_prelude::*, types::*,
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

impl DebugWithEngines for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: Engines<'_>) -> fmt::Result {
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
        let engines = Engines::new(ctx.type_engine, ctx.decl_engine);
        let possible = self.extract_any_including_self(engines, &filter_fn);
        let mut res = vec![];
        for type_id in possible.into_iter() {
            match ctx.type_engine.get(type_id) {
                TypeInfo::UnknownGeneric { name, .. } => {
                    panic!();
                    res.push(TypeMetadata::UnresolvedType(
                        name,
                        ctx.call_site_get(&type_id),
                    ));
                }
                TypeInfo::Placeholder(type_param) => {
                    panic!();
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
