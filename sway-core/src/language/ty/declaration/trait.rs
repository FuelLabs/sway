use std::hash::{Hash, Hasher};

use sway_types::{Ident, Span};

use crate::{
    decl_engine::DeclRef,
    engine_threading::*,
    language::{parsed, Visibility},
    transform,
    type_system::*,
};

#[derive(Clone, Debug)]
pub struct TyTraitDeclaration {
    pub name: Ident,
    pub type_parameters: Vec<TypeParam>,
    pub interface_surface: Vec<DeclRef>,
    pub methods: Vec<DeclRef>,
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
            && self.supertraits.eq(&other.supertraits, engines)
            && self.visibility == other.visibility
    }
}

impl HashWithEngines for TyTraitDeclaration {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TyTraitDeclaration {
            name,
            type_parameters,
            interface_surface,
            methods,
            supertraits,
            visibility,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            attributes: _,
            span: _,
        } = self;
        name.hash(state);
        type_parameters.hash(state, engines);
        interface_surface.hash(state, engines);
        methods.hash(state, engines);
        supertraits.hash(state, engines);
        visibility.hash(state);
    }
}

impl SubstTypes for TyTraitDeclaration {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
        self.interface_surface
            .iter_mut()
            .for_each(|function_decl_ref| {
                let new_decl_ref = function_decl_ref
                    .clone()
                    .subst_types_and_insert_new(type_mapping, engines);
                function_decl_ref.replace_id((&new_decl_ref).into());
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
            .for_each(|function_decl_ref| {
                let new_decl_ref = function_decl_ref
                    .clone()
                    .replace_self_type_and_insert_new(engines, self_type);
                function_decl_ref.replace_id((&new_decl_ref).into());
            });
        // we don't have to type check the methods because it hasn't been type checked yet
    }
}

impl MonomorphizeHelper for TyTraitDeclaration {
    fn name(&self) -> &Ident {
        &self.name
    }

    fn type_parameters(&self) -> &[TypeParam] {
        &self.type_parameters
    }
}
