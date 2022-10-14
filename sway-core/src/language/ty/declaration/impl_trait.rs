use sway_types::Span;

use crate::{declaration_engine::DeclarationId, language::CallPath, type_system::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TyImplTrait {
    pub trait_name: CallPath,
    pub(crate) span: Span,
    pub methods: Vec<DeclarationId>,
    pub implementing_for_type_id: TypeId,
    pub type_implementing_for_span: Span,
}

impl CopyTypes for TyImplTrait {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.methods
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
    }
}
