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

impl ReplaceSelfType for TypeId {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        fn helper(type_id: TypeId, engines: Engines<'_>, self_type: TypeId) -> Option<TypeId> {
            let type_engine = engines.te();
            let decl_engine = engines.de();
            match type_engine.get(type_id) {
                TypeInfo::SelfType => Some(self_type),
                TypeInfo::Enum {
                    type_parameters,
                    variant_types,
                    call_path,
                } => {
                    let mut need_to_create_new = false;
                    let variant_types = variant_types
                        .into_iter()
                        .map(|mut variant| {
                            if let Some(type_id) = helper(variant.type_id, engines, self_type) {
                                need_to_create_new = true;
                                variant.type_id = type_id;
                            }
                            variant
                        })
                        .collect::<Vec<_>>();
                    let type_parameters = type_parameters
                        .into_iter()
                        .map(|mut type_param| {
                            if let Some(type_id) = helper(type_param.type_id, engines, self_type) {
                                need_to_create_new = true;
                                type_param.type_id = type_id;
                            }
                            type_param
                        })
                        .collect::<Vec<_>>();
                    if need_to_create_new {
                        Some(type_engine.insert(
                            decl_engine,
                            TypeInfo::Enum {
                                variant_types,
                                type_parameters,
                                call_path,
                            },
                        ))
                    } else {
                        None
                    }
                }
                TypeInfo::Struct {
                    type_parameters,
                    fields,
                    call_path,
                } => {
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
                    let type_parameters = type_parameters
                        .into_iter()
                        .map(|mut type_param| {
                            if let Some(type_id) = helper(type_param.type_id, engines, self_type) {
                                need_to_create_new = true;
                                type_param.type_id = type_id;
                            }
                            type_param
                        })
                        .collect::<Vec<_>>();
                    if need_to_create_new {
                        Some(type_engine.insert(
                            decl_engine,
                            TypeInfo::Struct {
                                fields,
                                call_path,
                                type_parameters,
                            },
                        ))
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
                    name,
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
                                name,
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
                            if let Some(type_id) = helper(field.type_id, engines, self_type) {
                                need_to_create_new = true;
                                field.type_id = type_id;
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

    pub(crate) fn get_type_parameters(
        &self,
        type_engine: &TypeEngine,
    ) -> Option<Vec<TypeParameter>> {
        match type_engine.get(*self) {
            TypeInfo::Enum {
                type_parameters, ..
            } => (!type_parameters.is_empty()).then_some(type_parameters),
            TypeInfo::Struct {
                type_parameters, ..
            } => (!type_parameters.is_empty()).then_some(type_parameters),
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
                TypeInfo::Custom { name, .. },
                TypeInfo::Enum {
                    call_path: enum_call_path,
                    ..
                },
            ) => name != enum_call_path.suffix,
            (
                TypeInfo::Custom { name, .. },
                TypeInfo::Struct {
                    call_path: struct_call_path,
                    ..
                },
            ) => name != struct_call_path.suffix,
            (TypeInfo::Custom { .. }, _) => true,
            _ => false,
        }
    }
}
