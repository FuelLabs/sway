use crate::{
    decl_engine::{DeclMapping, MaterializeConstGenerics, ReplaceDecls},
    engine_threading::*,
    language::ty::{TyExpression, TyVariableDecl, VariableMutability},
    semantic_analysis::{
        TypeCheckAnalysis, TypeCheckAnalysisContext, TypeCheckContext, TypeCheckFinalization,
        TypeCheckFinalizationContext,
    },
    type_system::*,
    types::*,
    GenericArgument,
};
use ast_elements::type_parameter::ConstGenericExpr;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{Ident, Named, Span, Spanned};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum TyStatement {
}

impl EqWithEngines for TyStatement {}
impl PartialEqWithEngines for TyStatement {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        match (self, other) {
        }
    }
}

impl HashWithEngines for TyStatement {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        match self {
        }
    }
}

impl SubstTypes for TyStatement {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        match self {
        }
    }
}

impl ReplaceDecls for TyStatement {
    fn replace_decls_inner(
        &mut self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<bool, ErrorEmitted> {
        match self {
        }
    }
}

impl TypeCheckAnalysis for TyStatement {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        match self {
        }
        Ok(())
    }
}

impl TypeCheckFinalization for TyStatement {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        match self {
        }
        Ok(())
    }
}

impl CollectTypesMetadata for TyStatement {
    fn collect_types_metadata(
        &self,
        handler: &Handler,
        ctx: &mut CollectTypesMetadataContext,
    ) -> Result<Vec<TypeMetadata>, ErrorEmitted> {
        match self {
        }
    }
}

impl MaterializeConstGenerics for TyStatement {
    fn materialize_const_generics(
        &mut self,
        engines: &Engines,
        handler: &Handler,
        name: &str,
        value: &TyExpression,
    ) -> Result<(), ErrorEmitted> {
        match self {
        }
    }
}

