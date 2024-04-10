use std::hash::{Hash, Hasher};

use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::Ident;

use crate::{
    decl_engine::*,
    engine_threading::*,
    language::ty::*,
    semantic_analysis::{TypeCheckContext, TypeCheckFinalization, TypeCheckFinalizationContext},
    type_system::*,
};

#[derive(Clone, Debug)]
pub struct TyStructExpressionField {
    pub name: Ident,
    pub value: TyExpression,
}

impl EqWithEngines for TyStructExpressionField {}
impl PartialEqWithEngines for TyStructExpressionField {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name == other.name && self.value.eq(&other.value, ctx)
    }
}

impl HashWithEngines for TyStructExpressionField {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyStructExpressionField { name, value } = self;
        name.hash(state);
        value.hash(state, engines);
    }
}

impl SubstTypes for TyStructExpressionField {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> HasChanges {
        self.value.subst(type_mapping, engines)
    }
}

impl ReplaceDecls for TyStructExpressionField {
    fn replace_decls_inner(
        &mut self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<bool, ErrorEmitted> {
        self.value.replace_decls(decl_mapping, handler, ctx)
    }
}

impl TypeCheckFinalization for TyStructExpressionField {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        self.value.type_check_finalize(handler, ctx)
    }
}
