use derivative::Derivative;
use sway_types::{Ident, Span};

use crate::{
    declaration_engine::DeclarationId,
    language::{parsed, Visibility},
    transform,
    type_system::*,
};

#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq, Eq)]
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

impl CopyTypes for TyTraitDeclaration {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping) {
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
        self.interface_surface
            .iter_mut()
            .for_each(|function_decl_id| {
                let new_decl_id = function_decl_id
                    .clone()
                    .copy_types_and_insert_new(type_mapping);
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

impl ReplaceSelfType for TyTraitDeclaration {
    fn replace_self_type(&mut self, self_type: TypeId) {
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.replace_self_type(self_type));
        self.interface_surface
            .iter_mut()
            .for_each(|function_decl_id| {
                let new_decl_id = function_decl_id
                    .clone()
                    .replace_self_type_and_insert_new(self_type);
                function_decl_id.replace_id(*new_decl_id);
            });
        // we don't have to type check the methods because it hasn't been type checked yet
    }
}
