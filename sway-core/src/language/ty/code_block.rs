use crate::{language::ty::*, type_system::*, types::DeterministicallyAborts};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TyCodeBlock {
    pub contents: Vec<TyAstNode>,
}

impl CopyTypes for TyCodeBlock {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping) {
        self.contents
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
    }
}

impl ReplaceSelfType for TyCodeBlock {
    fn replace_self_type(&mut self, self_type: TypeId) {
        self.contents
            .iter_mut()
            .for_each(|x| x.replace_self_type(self_type));
    }
}

impl DeterministicallyAborts for TyCodeBlock {
    fn deterministically_aborts(&self) -> bool {
        self.contents.iter().any(|x| x.deterministically_aborts())
    }
}
