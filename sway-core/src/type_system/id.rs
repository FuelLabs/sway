#![allow(clippy::mutable_key_type)]
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{BaseIdent, Named, Span, Spanned};

use crate::{
    decl_engine::{
        DeclEngineGet, DeclEngineGetParsedDecl, DeclEngineInsert, MaterializeConstGenerics,
    },
    engine_threading::{DebugWithEngines, DisplayWithEngines, Engines, WithEngines},
    language::{ty::TyStructDecl, CallPath},
    namespace::TraitMap,
    semantic_analysis::TypeCheckContext,
    type_system::priv_prelude::*,
    types::{CollectTypesMetadata, CollectTypesMetadataContext, TypeMetadata},
    EnforceTypeArguments,
};

use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fmt,
};

use super::ast_elements::type_parameter::ConstGenericExpr;

const EXTRACT_ANY_MAX_DEPTH: usize = 128;

pub enum IncludeSelf {
    Yes,
    No,
}

pub enum TreatNumericAs {
    Abstract,
    Concrete,
}

/// A identifier to uniquely refer to our type terms
#[derive(PartialEq, Eq, Hash, Clone, Copy, Ord, PartialOrd, Debug, Deserialize, Serialize)]
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
            matches!(
                type_info,
                TypeInfo::UnknownGeneric { .. } | TypeInfo::Placeholder(_)
            )
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
                        type_param.name().clone(),
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
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        let type_engine = ctx.engines.te();
        if let Some(matching_id) = ctx
            .type_subst_map
            .and_then(|tsm| tsm.find_match(*self, ctx.engines))
        {
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

impl MaterializeConstGenerics for TypeId {
    fn materialize_const_generics(
        &mut self,
        engines: &Engines,
        handler: &Handler,
        name: &str,
        value: &crate::language::ty::TyExpression,
    ) -> Result<(), ErrorEmitted> {
        match &*engines.te().get(*self) {
            TypeInfo::Array(
                type_argument,
                Length(ConstGenericExpr::AmbiguousVariableExpression { ident }),
            ) if ident.as_str() == name => {
                let val = match &value.expression {
                    crate::language::ty::TyExpressionVariant::Literal(literal) => {
                        literal.cast_value_to_u64().unwrap()
                    }
                    _ => {
                        todo!("Will be implemented by https://github.com/FuelLabs/sway/issues/6860")
                    }
                };

                *self = engines.te().insert_array(
                    engines,
                    type_argument.clone(),
                    Length(ConstGenericExpr::Literal {
                        val: val as usize,
                        span: value.span.clone(),
                    }),
                );
            }
            TypeInfo::Enum(id) => {
                let decl = engines.de().get(id);
                let mut decl = (*decl).clone();
                decl.materialize_const_generics(engines, handler, name, value)?;

                let parsed_decl = engines
                    .de()
                    .get_parsed_decl(id)
                    .unwrap()
                    .to_enum_decl(handler, engines)
                    .ok();
                let decl_ref = engines.de().insert(decl, parsed_decl.as_ref());

                *self = engines.te().insert_enum(engines, *decl_ref.id());
            }
            TypeInfo::Struct(id) => {
                let mut decl = TyStructDecl::clone(&engines.de().get(id));
                decl.materialize_const_generics(engines, handler, name, value)?;

                let parsed_decl = engines
                    .de()
                    .get_parsed_decl(id)
                    .unwrap()
                    .to_struct_decl(handler, engines)
                    .ok();
                let decl_ref = engines.de().insert(decl, parsed_decl.as_ref());

                *self = engines.te().insert_struct(engines, *decl_ref.id());
            }
            TypeInfo::StringArray(Length(ConstGenericExpr::AmbiguousVariableExpression {
                ident,
            })) if ident.as_str() == name => {
                let val = match &value.expression {
                    crate::language::ty::TyExpressionVariant::Literal(literal) => {
                        literal.cast_value_to_u64().unwrap()
                    }
                    _ => {
                        todo!("Will be implemented by https://github.com/FuelLabs/sway/issues/6860")
                    }
                };

                *self = engines.te().insert_string_array(
                    engines,
                    Length(ConstGenericExpr::Literal {
                        val: val as usize,
                        span: value.span.clone(),
                    }),
                );
            }
            _ => {}
        }

        Ok(())
    }
}

impl TypeId {
    pub(super) const fn new(index: usize) -> TypeId {
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
            TypeInfo::Enum(decl_id) => {
                let decl = decl_engine.get(decl_id);
                (!decl.generic_parameters.is_empty()).then_some(decl.generic_parameters.clone())
            }
            TypeInfo::Struct(decl_ref) => {
                let decl = decl_engine.get_struct(decl_ref);
                (!decl.generic_parameters.is_empty()).then_some(decl.generic_parameters.clone())
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

    pub(crate) fn walk_any_including_self<F, WT, WTC>(
        self,
        engines: &Engines,
        filter_fn: &F,
        trait_constraints: Vec<TraitConstraint>,
        depth: usize,
        walk_type: &WT,
        walk_tc: &WTC,
    ) where
        F: Fn(&TypeInfo) -> bool,
        WT: Fn(&TypeId),
        WTC: Fn(&TraitConstraint),
    {
        let type_engine = engines.te();
        self.walk_any(engines, filter_fn, depth + 1, walk_type, walk_tc);
        let type_info = type_engine.get(self);
        if filter_fn(&type_info) {
            walk_type(&self);
            trait_constraints.iter().for_each(walk_tc);
        }
    }

    /// Returns all pairs of type parameters and its
    /// concrete types.
    /// This includes primitive types that have "implicit"
    /// type parameters such as tuples, arrays and others...
    pub(crate) fn extract_type_parameters(
        self,
        engines: &Engines,
        depth: usize,
        type_parameters: &mut Vec<(TypeId, TypeId)>,
        const_generic_parameters: &mut BTreeMap<String, crate::language::ty::TyExpression>,
        orig_type_id: TypeId,
    ) {
        if depth >= EXTRACT_ANY_MAX_DEPTH {
            panic!("Possible infinite recursion at extract_type_parameters");
        }

        let decl_engine = engines.de();
        match (&*engines.te().get(self), &*engines.te().get(orig_type_id)) {
            (TypeInfo::Unknown, TypeInfo::Unknown)
            | (TypeInfo::Never, TypeInfo::Never)
            | (TypeInfo::Placeholder(_), TypeInfo::Placeholder(_))
            | (TypeInfo::TypeParam(_), TypeInfo::TypeParam(_))
            | (TypeInfo::StringArray(_), TypeInfo::StringArray(_))
            | (TypeInfo::StringSlice, TypeInfo::StringSlice)
            | (TypeInfo::UnsignedInteger(_), TypeInfo::UnsignedInteger(_))
            | (TypeInfo::RawUntypedPtr, TypeInfo::RawUntypedPtr)
            | (TypeInfo::RawUntypedSlice, TypeInfo::RawUntypedSlice)
            | (TypeInfo::Boolean, TypeInfo::Boolean)
            | (TypeInfo::B256, TypeInfo::B256)
            | (TypeInfo::Numeric, TypeInfo::Numeric)
            | (TypeInfo::Contract, TypeInfo::Contract)
            | (TypeInfo::ErrorRecovery(_), TypeInfo::ErrorRecovery(_))
            | (TypeInfo::TraitType { .. }, TypeInfo::TraitType { .. }) => {}
            (TypeInfo::UntypedEnum(decl_id), TypeInfo::UntypedEnum(orig_decl_id)) => {
                let enum_decl = engines.pe().get_enum(decl_id);
                let orig_enum_decl = engines.pe().get_enum(orig_decl_id);
                assert_eq!(
                    enum_decl.type_parameters.len(),
                    orig_enum_decl.type_parameters.len()
                );
                for (type_param, orig_type_param) in enum_decl
                    .type_parameters
                    .iter()
                    .zip(orig_enum_decl.type_parameters.iter())
                {
                    let orig_type_param = orig_type_param
                        .as_type_parameter()
                        .expect("only works with type parameters");
                    let type_param = type_param
                        .as_type_parameter()
                        .expect("only works with type parameters");
                    type_parameters.push((type_param.type_id, orig_type_param.type_id));
                    type_param.type_id.extract_type_parameters(
                        engines,
                        depth + 1,
                        type_parameters,
                        const_generic_parameters,
                        orig_type_param.type_id,
                    );
                }
            }
            (TypeInfo::UntypedStruct(decl_id), TypeInfo::UntypedStruct(orig_decl_id)) => {
                let struct_decl = engines.pe().get_struct(decl_id);
                let orig_struct_decl = engines.pe().get_struct(orig_decl_id);
                assert_eq!(
                    struct_decl.type_parameters.len(),
                    orig_struct_decl.type_parameters.len()
                );
                for (type_param, orig_type_param) in struct_decl
                    .type_parameters
                    .iter()
                    .zip(orig_struct_decl.type_parameters.iter())
                {
                    let orig_type_param = orig_type_param
                        .as_type_parameter()
                        .expect("only works with type parameters");
                    let type_param = type_param
                        .as_type_parameter()
                        .expect("only works with type parameters");
                    type_parameters.push((type_param.type_id, orig_type_param.type_id));
                    type_param.type_id.extract_type_parameters(
                        engines,
                        depth + 1,
                        type_parameters,
                        const_generic_parameters,
                        orig_type_param.type_id,
                    );
                }
            }
            (TypeInfo::Enum(enum_ref), TypeInfo::Enum(orig_enum_ref)) => {
                let enum_decl = decl_engine.get_enum(enum_ref);
                let orig_enum_decl = decl_engine.get_enum(orig_enum_ref);
                assert_eq!(
                    enum_decl.generic_parameters.len(),
                    orig_enum_decl.generic_parameters.len()
                );
                for (type_param, orig_type_param) in enum_decl
                    .generic_parameters
                    .iter()
                    .zip(orig_enum_decl.generic_parameters.iter())
                {
                    match (orig_type_param, type_param) {
                        (TypeParameter::Type(orig_type_param), TypeParameter::Type(type_param)) => {
                            type_parameters.push((type_param.type_id, orig_type_param.type_id));
                            type_param.type_id.extract_type_parameters(
                                engines,
                                depth + 1,
                                type_parameters,
                                const_generic_parameters,
                                orig_type_param.type_id,
                            );
                        }
                        (
                            TypeParameter::Const(orig_type_param),
                            TypeParameter::Const(type_param),
                        ) => match (orig_type_param.expr.as_ref(), type_param.expr.as_ref()) {
                            (None, Some(expr)) => {
                                const_generic_parameters.insert(
                                    orig_type_param.name.as_str().to_string(),
                                    expr.to_ty_expression(engines),
                                );
                            }
                            _ => todo!("Will be implemented by https://github.com/FuelLabs/sway/issues/6860"),
                        },
                        _ => {}
                    }
                }
            }
            (TypeInfo::Struct(struct_id), TypeInfo::Struct(orig_struct_id)) => {
                let struct_decl = decl_engine.get_struct(struct_id);
                let orig_struct_decl = decl_engine.get_struct(orig_struct_id);
                assert_eq!(
                    struct_decl.generic_parameters.len(),
                    orig_struct_decl.generic_parameters.len()
                );
                for (type_param, orig_type_param) in struct_decl
                    .generic_parameters
                    .iter()
                    .zip(orig_struct_decl.generic_parameters.iter())
                {
                    match (orig_type_param, type_param) {
                        (TypeParameter::Type(orig_type_param), TypeParameter::Type(type_param)) => {
                            type_parameters.push((type_param.type_id, orig_type_param.type_id));
                            type_param.type_id.extract_type_parameters(
                                engines,
                                depth + 1,
                                type_parameters,
                                const_generic_parameters,
                                orig_type_param.type_id,
                            );
                        }
                        (
                            TypeParameter::Const(orig_type_param),
                            TypeParameter::Const(type_param),
                        ) => match (orig_type_param.expr.as_ref(), type_param.expr.as_ref()) {
                            (None, Some(expr)) => {
                                const_generic_parameters.insert(
                                    orig_type_param.name.as_str().to_string(),
                                    expr.to_ty_expression(engines),
                                );
                            }
                            _ => todo!("Will be implemented by https://github.com/FuelLabs/sway/issues/6860"),
                        },
                        _ => {}
                    }
                }
            }
            // Primitive types have "implicit" type parameters
            (TypeInfo::Tuple(elems), TypeInfo::Tuple(orig_elems)) => {
                assert_eq!(elems.len(), orig_elems.len());
                for (elem, orig_elem) in elems.iter().zip(orig_elems.iter()) {
                    type_parameters.push((elem.type_id(), orig_elem.type_id()));
                    elem.type_id().extract_type_parameters(
                        engines,
                        depth + 1,
                        type_parameters,
                        const_generic_parameters,
                        orig_elem.type_id(),
                    );
                }
            }
            (
                TypeInfo::ContractCaller {
                    abi_name: _,
                    address,
                },
                TypeInfo::ContractCaller {
                    abi_name: _,
                    address: orig_address,
                },
            ) => {
                if let Some(address) = address {
                    address.return_type.extract_type_parameters(
                        engines,
                        depth + 1,
                        type_parameters,
                        const_generic_parameters,
                        orig_address.clone().unwrap().return_type,
                    );
                }
            }
            (
                TypeInfo::Custom {
                    qualified_call_path: _,
                    type_arguments,
                },
                TypeInfo::Custom {
                    qualified_call_path: _,
                    type_arguments: orig_type_arguments,
                },
            ) => {
                if let Some(type_arguments) = type_arguments {
                    for (type_arg, orig_type_arg) in type_arguments
                        .iter()
                        .zip(orig_type_arguments.clone().unwrap().iter())
                    {
                        type_arg.type_id().extract_type_parameters(
                            engines,
                            depth + 1,
                            type_parameters,
                            const_generic_parameters,
                            orig_type_arg.type_id(),
                        );
                    }
                }
            }
            // Primitive types have "implicit" type parameters
            (TypeInfo::Array(ty, _), TypeInfo::Array(orig_ty, _)) => {
                type_parameters.push((ty.type_id(), orig_ty.type_id()));
                ty.type_id().extract_type_parameters(
                    engines,
                    depth + 1,
                    type_parameters,
                    const_generic_parameters,
                    orig_ty.type_id(),
                );
            }
            (TypeInfo::Alias { name: _, ty }, _) => {
                ty.type_id().extract_type_parameters(
                    engines,
                    depth + 1,
                    type_parameters,
                    const_generic_parameters,
                    orig_type_id,
                );
            }
            (_, TypeInfo::Alias { name: _, ty }) => {
                self.extract_type_parameters(
                    engines,
                    depth + 1,
                    type_parameters,
                    const_generic_parameters,
                    ty.type_id(),
                );
            }
            (TypeInfo::UnknownGeneric { .. }, TypeInfo::UnknownGeneric { .. }) => {}
            // Primitive types have "implicit" type parameters
            (TypeInfo::Ptr(ty), TypeInfo::Ptr(orig_ty)) => {
                type_parameters.push((ty.type_id(), orig_ty.type_id()));
                ty.type_id().extract_type_parameters(
                    engines,
                    depth + 1,
                    type_parameters,
                    const_generic_parameters,
                    orig_ty.type_id(),
                );
            }
            // Primitive types have "implicit" type parameters
            (TypeInfo::Slice(ty), TypeInfo::Slice(orig_ty)) => {
                type_parameters.push((ty.type_id(), orig_ty.type_id()));
                ty.type_id().extract_type_parameters(
                    engines,
                    depth + 1,
                    type_parameters,
                    const_generic_parameters,
                    orig_ty.type_id(),
                );
            }
            // Primitive types have "implicit" type parameters
            (
                TypeInfo::Ref {
                    referenced_type, ..
                },
                TypeInfo::Ref {
                    referenced_type: orig_referenced_type,
                    ..
                },
            ) => {
                type_parameters.push((referenced_type.type_id(), orig_referenced_type.type_id()));
                referenced_type.type_id().extract_type_parameters(
                    engines,
                    depth + 1,
                    type_parameters,
                    const_generic_parameters,
                    orig_referenced_type.type_id(),
                );
            }
            (_, TypeInfo::UnknownGeneric { .. }) => {}
            (_, _) => {}
        }
    }

    pub(crate) fn walk_any<F, WT, WTC>(
        self,
        engines: &Engines,
        filter_fn: &F,
        depth: usize,
        walk_type: &WT,
        walk_tc: &WTC,
    ) where
        F: Fn(&TypeInfo) -> bool,
        WT: Fn(&TypeId),
        WTC: Fn(&TraitConstraint),
    {
        if depth >= EXTRACT_ANY_MAX_DEPTH {
            panic!("Possible infinite recursion at walk_any");
        }

        let decl_engine = engines.de();
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
            TypeInfo::UntypedEnum(decl_id) => {
                let enum_decl = engines.pe().get_enum(decl_id);
                for type_param in &enum_decl.type_parameters {
                    match type_param {
                        TypeParameter::Type(type_param) => {
                            type_param.type_id.walk_any_including_self(
                                engines,
                                filter_fn,
                                type_param.trait_constraints.clone(),
                                depth + 1,
                                walk_type,
                                walk_tc,
                            )
                        }
                        TypeParameter::Const(type_param) => type_param.ty.walk_any_including_self(
                            engines,
                            filter_fn,
                            vec![],
                            depth + 1,
                            walk_type,
                            walk_tc,
                        ),
                    }
                }
                for variant in &enum_decl.variants {
                    variant.type_argument.type_id().walk_any_including_self(
                        engines,
                        filter_fn,
                        vec![],
                        depth + 1,
                        walk_type,
                        walk_tc,
                    );
                }
            }
            TypeInfo::UntypedStruct(decl_id) => {
                let struct_decl = engines.pe().get_struct(decl_id);
                for type_param in &struct_decl.type_parameters {
                    match type_param {
                        TypeParameter::Type(type_param) => {
                            type_param.type_id.walk_any_including_self(
                                engines,
                                filter_fn,
                                type_param.trait_constraints.clone(),
                                depth + 1,
                                walk_type,
                                walk_tc,
                            )
                        }
                        TypeParameter::Const(type_param) => type_param.ty.walk_any_including_self(
                            engines,
                            filter_fn,
                            vec![],
                            depth + 1,
                            walk_type,
                            walk_tc,
                        ),
                    }
                }
                for field in &struct_decl.fields {
                    field.type_argument.type_id().walk_any_including_self(
                        engines,
                        filter_fn,
                        vec![],
                        depth + 1,
                        walk_type,
                        walk_tc,
                    );
                }
            }
            TypeInfo::Enum(enum_ref) => {
                let enum_decl = decl_engine.get_enum(enum_ref);
                for type_param in &enum_decl.generic_parameters {
                    match type_param {
                        TypeParameter::Type(type_param) => {
                            type_param.type_id.walk_any_including_self(
                                engines,
                                filter_fn,
                                type_param.trait_constraints.clone(),
                                depth + 1,
                                walk_type,
                                walk_tc,
                            )
                        }
                        TypeParameter::Const(type_param) => type_param.ty.walk_any_including_self(
                            engines,
                            filter_fn,
                            vec![],
                            depth + 1,
                            walk_type,
                            walk_tc,
                        ),
                    }
                }
                for variant in &enum_decl.variants {
                    variant.type_argument.type_id().walk_any_including_self(
                        engines,
                        filter_fn,
                        vec![],
                        depth + 1,
                        walk_type,
                        walk_tc,
                    );
                }
            }
            TypeInfo::Struct(struct_id) => {
                let struct_decl = decl_engine.get_struct(struct_id);
                for type_param in &struct_decl.generic_parameters {
                    match type_param {
                        TypeParameter::Type(type_param) => {
                            type_param.type_id.walk_any_including_self(
                                engines,
                                filter_fn,
                                type_param.trait_constraints.clone(),
                                depth + 1,
                                walk_type,
                                walk_tc,
                            )
                        }
                        TypeParameter::Const(type_param) => type_param.ty.walk_any_including_self(
                            engines,
                            filter_fn,
                            vec![],
                            depth + 1,
                            walk_type,
                            walk_tc,
                        ),
                    }
                }
                for field in &struct_decl.fields {
                    field.type_argument.type_id().walk_any_including_self(
                        engines,
                        filter_fn,
                        vec![],
                        depth + 1,
                        walk_type,
                        walk_tc,
                    );
                }
            }
            TypeInfo::Tuple(elems) => {
                for elem in elems {
                    elem.type_id().walk_any_including_self(
                        engines,
                        filter_fn,
                        vec![],
                        depth + 1,
                        walk_type,
                        walk_tc,
                    );
                }
            }
            TypeInfo::ContractCaller {
                abi_name: _,
                address,
            } => {
                if let Some(address) = address {
                    address.return_type.walk_any_including_self(
                        engines,
                        filter_fn,
                        vec![],
                        depth + 1,
                        walk_type,
                        walk_tc,
                    );
                }
            }
            TypeInfo::Custom {
                qualified_call_path: _,
                type_arguments,
            } => {
                if let Some(type_arguments) = type_arguments {
                    for type_arg in type_arguments {
                        type_arg.type_id().walk_any_including_self(
                            engines,
                            filter_fn,
                            vec![],
                            depth + 1,
                            walk_type,
                            walk_tc,
                        );
                    }
                }
            }
            TypeInfo::Array(ty, _) => {
                ty.type_id().walk_any_including_self(
                    engines,
                    filter_fn,
                    vec![],
                    depth + 1,
                    walk_type,
                    walk_tc,
                );
            }
            TypeInfo::Alias { name: _, ty } => {
                ty.type_id().walk_any_including_self(
                    engines,
                    filter_fn,
                    vec![],
                    depth + 1,
                    walk_type,
                    walk_tc,
                );
            }
            TypeInfo::UnknownGeneric {
                name: _,
                trait_constraints,
                parent: _,
                is_from_type_parameter: _,
            } => {
                walk_type(&self);
                for trait_constraint in trait_constraints.iter() {
                    for type_arg in &trait_constraint.type_arguments {
                        // In case type_id was already added skip it.
                        // This is required because of recursive generic trait such as `T: Trait<T>`
                        type_arg.type_id().walk_any_including_self(
                            engines,
                            filter_fn,
                            vec![],
                            depth + 1,
                            walk_type,
                            walk_tc,
                        );
                    }
                }
            }
            TypeInfo::Ptr(ty) => {
                ty.type_id().walk_any_including_self(
                    engines,
                    filter_fn,
                    vec![],
                    depth + 1,
                    walk_type,
                    walk_tc,
                );
            }
            TypeInfo::Slice(ty) => {
                ty.type_id().walk_any_including_self(
                    engines,
                    filter_fn,
                    vec![],
                    depth + 1,
                    walk_type,
                    walk_tc,
                );
            }
            TypeInfo::Ref {
                referenced_type, ..
            } => {
                referenced_type.type_id().walk_any_including_self(
                    engines,
                    filter_fn,
                    vec![],
                    depth + 1,
                    walk_type,
                    walk_tc,
                );
            }
        }
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
            TypeInfo::UntypedEnum(decl_id) => {
                let enum_decl = engines.pe().get_enum(decl_id);
                for type_param in &enum_decl.type_parameters {
                    let type_param = type_param
                        .as_type_parameter()
                        .expect("only works with type parameters");
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
                        variant.type_argument.type_id().extract_any_including_self(
                            engines,
                            filter_fn,
                            vec![],
                            depth + 1,
                        ),
                    );
                }
            }
            TypeInfo::UntypedStruct(decl_id) => {
                let struct_decl = engines.pe().get_struct(decl_id);
                for type_param in &struct_decl.type_parameters {
                    let type_param = type_param
                        .as_type_parameter()
                        .expect("only works with type parameters");
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
                        field.type_argument.type_id().extract_any_including_self(
                            engines,
                            filter_fn,
                            vec![],
                            depth + 1,
                        ),
                    );
                }
            }
            TypeInfo::Enum(enum_ref) => {
                let enum_decl = decl_engine.get_enum(enum_ref);
                let type_params = enum_decl
                    .generic_parameters
                    .iter()
                    .filter_map(|x| x.as_type_parameter());
                for type_param in type_params {
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
                        variant.type_argument.type_id().extract_any_including_self(
                            engines,
                            filter_fn,
                            vec![],
                            depth + 1,
                        ),
                    );
                }
            }
            TypeInfo::Struct(struct_id) => {
                let struct_decl = decl_engine.get_struct(struct_id);
                let type_params = struct_decl
                    .generic_parameters
                    .iter()
                    .filter_map(|x| x.as_type_parameter());
                for type_param in type_params {
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
                        field.type_argument.type_id().extract_any_including_self(
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
                        elem.type_id().extract_any_including_self(
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
            } => {
                if let Some(type_arguments) = type_arguments {
                    for type_arg in type_arguments {
                        extend(
                            &mut found,
                            type_arg.type_id().extract_any_including_self(
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
                    ty.type_id()
                        .extract_any_including_self(engines, filter_fn, vec![], depth + 1),
                );
            }
            TypeInfo::Alias { name: _, ty } => {
                extend(
                    &mut found,
                    ty.type_id()
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
                        if !found.contains_key(&type_arg.type_id()) {
                            extend(
                                &mut found,
                                type_arg.type_id().extract_any_including_self(
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
                    ty.type_id()
                        .extract_any_including_self(engines, filter_fn, vec![], depth + 1),
                );
            }
            TypeInfo::Slice(ty) => {
                extend(
                    &mut found,
                    ty.type_id()
                        .extract_any_including_self(engines, filter_fn, vec![], depth + 1),
                );
            }
            TypeInfo::Ref {
                referenced_type, ..
            } => {
                extend(
                    &mut found,
                    referenced_type.type_id().extract_any_including_self(
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
    /// `TypeId`'s of `self`.
    pub(crate) fn extract_inner_types(
        &self,
        engines: &Engines,
        include_self: IncludeSelf,
    ) -> BTreeSet<TypeId> {
        let mut set: BTreeSet<TypeId> = self
            .extract_any(engines, &|_| true, 0)
            .keys()
            .copied()
            .collect();

        if matches!(include_self, IncludeSelf::Yes) {
            set.insert(*self);
        }

        set
    }

    /// Given a `TypeId` `self`, analyze `self` and return all inner
    /// `TypeId`'s of `self`.
    pub(crate) fn walk_inner_types<WT, WTC>(
        &self,
        engines: &Engines,
        include_self: IncludeSelf,
        walk_type: &WT,
        walk_tc: &WTC,
    ) where
        WT: Fn(&TypeId),
        WTC: Fn(&TraitConstraint),
    {
        self.walk_any(engines, &|_| true, 0, walk_type, walk_tc);

        if matches!(include_self, IncludeSelf::Yes) {
            walk_type(self);
        }
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
            .extract_inner_types(engines, IncludeSelf::No)
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

    pub(crate) fn is_concrete(&self, engines: &Engines, treat_numeric_as: TreatNumericAs) -> bool {
        let nested_types = (*self).extract_nested_types(engines);
        !nested_types.into_iter().any(|x| match treat_numeric_as {
            TreatNumericAs::Abstract => matches!(
                x,
                TypeInfo::UnknownGeneric { .. }
                    | TypeInfo::Custom { .. }
                    | TypeInfo::Placeholder(..)
                    | TypeInfo::TraitType { .. }
                    | TypeInfo::TypeParam(..)
                    | TypeInfo::Numeric
            ),
            TreatNumericAs::Concrete => matches!(
                x,
                TypeInfo::UnknownGeneric { .. }
                    | TypeInfo::Custom { .. }
                    | TypeInfo::Placeholder(..)
                    | TypeInfo::TraitType { .. }
                    | TypeInfo::TypeParam(..)
            ),
        })
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
        if ctx.code_block_first_pass() {
            return Ok(());
        }

        let engines = ctx.engines();

        let mut structure_generics = self.extract_inner_types_with_trait_constraints(engines);

        if let Some(type_param) = type_param {
            match type_param {
                TypeParameter::Type(p) => {
                    structure_generics.insert(self, p.trait_constraints);
                }
                TypeParameter::Const(_) => {
                    todo!("Will be implemented by https://github.com/FuelLabs/sway/issues/6860")
                }
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
            Ok(())
        })
    }

    fn check_trait_constraints_errors(
        self,
        handler: &Handler,
        ctx: TypeCheckContext,
        structure_type_id: &TypeId,
        structure_trait_constraints: &Vec<TraitConstraint>,
        f: impl Fn(&TraitConstraint),
    ) -> bool {
        let engines = ctx.engines();

        let unify_check = UnifyCheck::constraint_subset(engines);
        let mut found_error = false;
        let generic_trait_constraints_trait_names_and_args =
            TraitMap::get_trait_names_and_type_arguments_for_type(
                ctx.namespace().current_module(),
                engines,
                *structure_type_id,
            );
        for structure_trait_constraint in structure_trait_constraints {
            let structure_trait_constraint_trait_name = &structure_trait_constraint
                .trait_name
                .to_canonical_path(ctx.engines(), ctx.namespace());

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
                                        t1.type_id(),
                                        &t1.span(),
                                        EnforceTypeArguments::No,
                                        None,
                                    )
                                    .unwrap_or_else(|err| engines.te().id_of_error_recovery(err)),
                                    ctx.resolve_type(
                                        handler,
                                        t2.type_id(),
                                        &t2.span(),
                                        EnforceTypeArguments::No,
                                        None,
                                    )
                                    .unwrap_or_else(|err| engines.te().id_of_error_recovery(err)),
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

    pub fn get_type_str(&self, engines: &Engines) -> String {
        engines.te().get(*self).get_type_str(engines)
    }
}
