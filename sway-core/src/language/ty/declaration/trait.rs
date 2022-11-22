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
    fn eq(&self, rhs: &Self, type_engine: &TypeEngine) -> bool {
        self.name == rhs.name
            && self.type_parameters.eq(&rhs.type_parameters, type_engine)
            && self
                .interface_surface
                .eq(&rhs.interface_surface, type_engine)
            && self.methods.eq(&rhs.methods, type_engine)
            && self.supertraits == rhs.supertraits
            && self.visibility == rhs.visibility
            && self.attributes == rhs.attributes
            && self.span == rhs.span
    }
}

impl CopyTypes for TyTraitDeclaration {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, type_engine: &TypeEngine) {
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping, type_engine));
        self.interface_surface
            .iter_mut()
            .for_each(|function_decl_id| {
                let new_decl_id = function_decl_id
                    .clone()
                    .copy_types_and_insert_new(type_mapping, type_engine);
                function_decl_id.replace_id(*new_decl_id);
            });
        // we don't have to type check the methods because it hasn't been type checked yet
    }
}

impl ReplaceSelfType for TyTraitDeclaration {
    fn replace_self_type(&mut self, type_engine: &TypeEngine, self_type: TypeId) {
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.replace_self_type(type_engine, self_type));
        self.interface_surface
            .iter_mut()
            .for_each(|function_decl_id| {
                let new_decl_id = function_decl_id
                    .clone()
                    .replace_self_type_and_insert_new(type_engine, self_type);
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
