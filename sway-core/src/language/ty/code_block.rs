use std::hash::Hasher;

use crate::{decl_engine::*, engine_threading::*, language::ty::*, types::DeterministicallyAborts};

#[derive(Clone, Debug)]
pub struct TyCodeBlock {
    pub contents: Vec<TyAstNode>,
}

impl EqWithEngines for TyCodeBlock {}
impl PartialEqWithEngines for TyCodeBlock {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.contents.eq(&other.contents, engines)
    }
}

impl HashWithEngines for TyCodeBlock {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TyCodeBlock { contents } = self;
        contents.hash(state, engines);
    }
}

impl DeterministicallyAborts for TyCodeBlock {
    fn deterministically_aborts(&self, decl_engine: &DeclEngine, check_call_body: bool) -> bool {
        self.contents
            .iter()
            .any(|x| x.deterministically_aborts(decl_engine, check_call_body))
    }
}
