#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TyCodeBlock {
    pub contents: Vec<TyAstNode>,
}

impl CopyTypes for TyCodeBlock {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.contents
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
    }
}

impl DeterministicallyAborts for TyCodeBlock {
    fn deterministically_aborts(&self) -> bool {
        self.contents.iter().any(|x| x.deterministically_aborts())
    }
}
