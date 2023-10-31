use std::hash::Hasher;

use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::Span;

use crate::{
    decl_engine::*, engine_threading::*, language::ty::*, semantic_analysis::TypeCheckContext,
    type_system::*, types::DeterministicallyAborts,
};

#[derive(Clone, Debug)]
pub struct TyCodeBlock {
    pub contents: Vec<TyAstNode>,
    pub(crate) whole_block_span: Span,
}

impl Default for TyCodeBlock {
    fn default() -> Self {
        Self {
            contents: Default::default(),
            whole_block_span: Span::dummy(),
        }
    }
}

impl EqWithEngines for TyCodeBlock {}
impl PartialEqWithEngines for TyCodeBlock {
    fn eq(&self, other: &Self, engines: &Engines) -> bool {
        self.contents.eq(&other.contents, engines)
    }
}

impl HashWithEngines for TyCodeBlock {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyCodeBlock { contents, .. } = self;
        contents.hash(state, engines);
    }
}

impl SubstTypes for TyCodeBlock {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) {
        self.contents
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
    }
}

impl ReplaceDecls for TyCodeBlock {
    fn replace_decls_inner(
        &mut self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<(), ErrorEmitted> {
        handler.scope(|handler| {
            for x in self.contents.iter_mut() {
                match x.replace_decls(decl_mapping, handler, ctx) {
                    Ok(res) => res,
                    Err(_) => {
                        continue;
                    }
                };
            }

            Ok(())
        })
    }
}

impl UpdateConstantExpression for TyCodeBlock {
    fn update_constant_expression(&mut self, engines: &Engines, implementing_type: &TyDecl) {
        self.contents
            .iter_mut()
            .for_each(|x| x.update_constant_expression(engines, implementing_type));
    }
}

impl DeterministicallyAborts for TyCodeBlock {
    fn deterministically_aborts(&self, decl_engine: &DeclEngine, check_call_body: bool) -> bool {
        self.contents
            .iter()
            .any(|x| x.deterministically_aborts(decl_engine, check_call_body))
    }
}
