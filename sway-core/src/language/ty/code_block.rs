use crate::{
    declaration_engine::{DeclMapping, ReplaceDecls},
    engine_threading::*,
    language::ty::*,
    type_system::*,
    types::DeterministicallyAborts,
};

#[derive(Clone, Debug)]
pub struct TyCodeBlock {
    pub contents: Vec<TyAstNode>,
}

impl EqWithEngines for TyCodeBlock {}
impl PartialEqWithEngines for TyCodeBlock {
    fn eq(&self, rhs: &Self, type_engine: &TypeEngine) -> bool {
        self.contents.eq(&rhs.contents, type_engine)
    }
}

impl CopyTypes for TyCodeBlock {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, type_engine: &TypeEngine) {
        self.contents
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping, type_engine));
    }
}

impl ReplaceSelfType for TyCodeBlock {
    fn replace_self_type(&mut self, type_engine: &TypeEngine, self_type: TypeId) {
        self.contents
            .iter_mut()
            .for_each(|x| x.replace_self_type(type_engine, self_type));
    }
}

impl ReplaceDecls for TyCodeBlock {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping, type_engine: &TypeEngine) {
        self.contents
            .iter_mut()
            .for_each(|x| x.replace_decls(decl_mapping, type_engine));
    }
}

impl DeterministicallyAborts for TyCodeBlock {
    fn deterministically_aborts(&self, check_call_body: bool) -> bool {
        self.contents
            .iter()
            .any(|x| x.deterministically_aborts(check_call_body))
    }
}
