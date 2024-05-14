use std::hash::{Hash, Hasher};

use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::Ident;

use crate::{
    engine_threading::*,
    language::ty::*,
    semantic_analysis::{
        TypeCheckAnalysis, TypeCheckAnalysisContext, TypeCheckFinalization,
        TypeCheckFinalizationContext,
    },
    type_system::*,
};

#[derive(Clone, Debug)]
pub struct TyVariableDecl {
    pub name: Ident,
    pub body: TyExpression,
    pub mutability: VariableMutability,
    pub return_type: TypeId,
    pub type_ascription: TypeArgument,
}

impl EqWithEngines for TyVariableDecl {}
impl PartialEqWithEngines for TyVariableDecl {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let type_engine = ctx.engines().te();
        self.name == other.name
            && self.body.eq(&other.body, ctx)
            && self.mutability == other.mutability
            && type_engine
                .get(self.return_type)
                .eq(&type_engine.get(other.return_type), ctx)
            && self.type_ascription.eq(&other.type_ascription, ctx)
    }
}

impl HashWithEngines for TyVariableDecl {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyVariableDecl {
            name,
            body,
            mutability,
            return_type,
            type_ascription,
        } = self;
        let type_engine = engines.te();
        name.hash(state);
        body.hash(state, engines);
        type_engine.get(*return_type).hash(state, engines);
        type_ascription.hash(state, engines);
        mutability.hash(state);
    }
}

impl SubstTypes for TyVariableDecl {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> HasChanges {
        self.return_type.subst(type_mapping, engines);
        self.type_ascription.subst(type_mapping, engines);
        self.body.subst(type_mapping, engines)
    }
}

impl TypeCheckAnalysis for TyVariableDecl {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        self.body.type_check_analyze(handler, ctx)?;
        Ok(())
    }
}

impl TypeCheckFinalization for TyVariableDecl {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        self.body.type_check_finalize(handler, ctx)
    }
}
