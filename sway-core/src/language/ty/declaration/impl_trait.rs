use sway_types::Span;

use crate::{declaration_engine::DeclarationId, language::CallPath, type_system::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TyImplTrait {
    pub impl_type_parameters: Vec<TypeParameter>,
    pub trait_name: CallPath,
    pub trait_type_arguments: Vec<TypeArgument>,
    pub methods: Vec<DeclarationId>,
    pub implementing_for_type_id: TypeId,
    pub type_implementing_for_span: Span,
    pub span: Span,
}

impl CopyTypes for TyImplTrait {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, type_engine: &TypeEngine) {
        self.impl_type_parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping, type_engine));
        self.implementing_for_type_id
            .copy_types(type_mapping, type_engine);
        self.methods
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping, type_engine));
    }
}

impl ReplaceSelfType for TyImplTrait {
    fn replace_self_type(&mut self, type_engine: &TypeEngine, self_type: TypeId) {
        self.impl_type_parameters
            .iter_mut()
            .for_each(|x| x.replace_self_type(type_engine, self_type));
        self.implementing_for_type_id
            .replace_self_type(type_engine, self_type);
        self.methods
            .iter_mut()
            .for_each(|x| x.replace_self_type(type_engine, self_type));
    }
}
