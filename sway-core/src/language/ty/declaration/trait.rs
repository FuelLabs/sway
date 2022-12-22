use sway_types::{Ident, Span};

use crate::{
    declaration_engine::DeclarationId,
    engine_threading::*,
    language::{parsed, Visibility},
    transform,
    type_system::*,
};

#[derive(Clone, Debug)]
pub struct TyTraitDeclaration {
    pub name: Ident,
    pub type_parameters: Vec<TypeParameter>,
    pub interface_surface: Vec<DeclarationId>,
    pub methods: Vec<DeclarationId>,
    pub supertraits: Vec<parsed::Supertrait>,
    pub visibility: Visibility,
    pub attributes: transform::AttributesMap,
    pub span: Span,
}

impl EqWithEngines for TyTraitDeclaration {}
impl PartialEqWithEngines for TyTraitDeclaration {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.name == other.name
            && self.type_parameters.eq(&other.type_parameters, engines)
            && self.interface_surface.eq(&other.interface_surface, engines)
            && self.methods.eq(&other.methods, engines)
            && self.supertraits == other.supertraits
            && self.visibility == other.visibility
            && self.attributes == other.attributes
            && self.span == other.span
    }
}

impl CopyTypes for TyTraitDeclaration {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, engines: Engines<'_>) {
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping, engines));
        self.interface_surface
            .iter_mut()
            .for_each(|function_decl_id| {
                let new_decl_id = function_decl_id
                    .clone()
                    .copy_types_and_insert_new(type_mapping, engines);
                function_decl_id.replace_id(*new_decl_id);
            });
        // we don't have to type check the methods because it hasn't been type checked yet
    }
}

impl ReplaceSelfType for TyTraitDeclaration {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.replace_self_type(engines, self_type));
        self.interface_surface
            .iter_mut()
            .for_each(|function_decl_id| {
                let new_decl_id = function_decl_id
                    .clone()
                    .replace_self_type_and_insert_new(engines, self_type);
                function_decl_id.replace_id(*new_decl_id);
            });
        // we don't have to type check the methods because it hasn't been type checked yet
    }
}

impl MonomorphizeHelper for TyTraitDeclaration {
    fn name(&self) -> &Ident {
        &self.name
    }

    fn type_parameters(&self) -> &[TypeParameter] {
        &self.type_parameters
    }
}
