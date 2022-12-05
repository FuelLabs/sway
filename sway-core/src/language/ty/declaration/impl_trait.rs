use sway_types::Span;

use crate::{
    declaration_engine::DeclarationId, engine_threading::*, language::CallPath, type_system::*,
};

#[derive(Clone, Debug)]
pub struct TyImplTrait {
    pub impl_type_parameters: Vec<TypeParameter>,
    pub trait_name: CallPath,
    pub trait_type_arguments: Vec<TypeArgument>,
    pub methods: Vec<DeclarationId>,
    pub implementing_for_type_id: TypeId,
    pub type_implementing_for_span: Span,
    pub span: Span,
}

impl EqWithEngines for TyImplTrait {}
impl PartialEqWithEngines for TyImplTrait {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.impl_type_parameters
            .eq(&other.impl_type_parameters, engines)
            && self.trait_name == other.trait_name
            && self
                .trait_type_arguments
                .eq(&other.trait_type_arguments, engines)
            && self.methods.eq(&other.methods, engines)
            && self.implementing_for_type_id == other.implementing_for_type_id
            && self.type_implementing_for_span == other.type_implementing_for_span
            && self.span == other.span
    }
}

impl CopyTypes for TyImplTrait {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, engines: Engines<'_>) {
        self.impl_type_parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping, engines));
        self.implementing_for_type_id
            .copy_types(type_mapping, engines);
        self.methods
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping, engines));
    }
}

impl ReplaceSelfType for TyImplTrait {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.impl_type_parameters
            .iter_mut()
            .for_each(|x| x.replace_self_type(engines, self_type));
        self.implementing_for_type_id
            .replace_self_type(engines, self_type);
        self.methods
            .iter_mut()
            .for_each(|x| x.replace_self_type(engines, self_type));
    }
}
