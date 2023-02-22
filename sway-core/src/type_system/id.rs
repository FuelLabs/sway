use super::*;
use crate::engine_threading::*;

use std::fmt;

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
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut res = vec![];
        match ctx.type_engine.get(*self) {
            TypeInfo::UnknownGeneric {
                name,
                trait_constraints,
            } => {
                res.push(TypeMetadata::UnresolvedType(name, ctx.call_site_get(self)));
                for trait_constraint in trait_constraints.iter() {
                    res.extend(check!(
                        trait_constraint.collect_types_metadata(ctx),
                        continue,
                        warnings,
                        errors
                    ));
                }
            }
            TypeInfo::Placeholder(type_param) => {
                res.push(TypeMetadata::UnresolvedType(
                    type_param.name_ident,
                    ctx.call_site_get(self),
                ));
            }
            _ => {}
        }
        if let TypeInfo::UnknownGeneric {
            name,
            trait_constraints,
        } = ctx.type_engine.get(*self)
        {
            res.push(TypeMetadata::UnresolvedType(name, ctx.call_site_get(self)));
            for trait_constraint in trait_constraints.iter() {
                res.extend(check!(
                    trait_constraint.collect_types_metadata(ctx),
                    continue,
                    warnings,
                    errors
                ));
            }
        }
        if errors.is_empty() {
            ok(res, warnings, errors)
        } else {
            err(warnings, errors)
        }
    }
}

impl SubstTypes for TypeId {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        if let Some(matching_id) = type_mapping.find_match(*self, engines) {
            *self = matching_id;
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
        type_engine
            .get(*self)
            .type_parameter_is_unconstrained(engines, type_parameter)
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

    pub(crate) fn get_type_parameters(&self, type_engine: &TypeEngine) -> Option<TypeParameters> {
        match type_engine.get(*self) {
            TypeInfo::Enum {
                type_parameters, ..
            } => (!type_parameters.is_empty_excluding_self()).then_some(type_parameters),
            TypeInfo::Struct {
                type_parameters, ..
            } => (!type_parameters.is_empty_excluding_self()).then_some(type_parameters),
            _ => None,
        }
    }

    /// Indicates of a given type is generic or not. Rely on whether the type is `Custom` and
    /// consider the special case where the resolved type is a struct or enum with a name that
    /// matches the name of the `Custom`.
    pub(crate) fn is_generic_parameter(
        self,
        type_engine: &TypeEngine,
        resolved_type_id: TypeId,
    ) -> bool {
        match (type_engine.get(self), type_engine.get(resolved_type_id)) {
            (
                TypeInfo::Custom { call_path, .. },
                TypeInfo::Enum {
                    call_path: enum_call_path,
                    ..
                },
            ) => call_path.suffix != enum_call_path.suffix,
            (
                TypeInfo::Custom { call_path, .. },
                TypeInfo::Struct {
                    call_path: struct_call_path,
                    ..
                },
            ) => call_path.suffix != struct_call_path.suffix,
            (TypeInfo::Custom { .. }, _) => true,
            _ => false,
        }
    }
}
