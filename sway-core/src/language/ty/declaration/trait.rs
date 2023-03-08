use std::hash::{Hash, Hasher};

use sway_types::{Ident, Named, Span, Spanned};

use crate::{
    decl_engine::{DeclRefFunction, DeclRefTraitFn, ReplaceFunctionImplementingType},
    engine_threading::*,
    language::{parsed, Visibility},
    transform,
    type_system::*,
};

use super::TyDeclaration;

#[derive(Clone, Debug)]
pub struct TyTraitDeclaration {
    pub name: Ident,
    pub type_parameters: Vec<TypeParameter>,
    pub interface_surface: Vec<TyTraitInterfaceItem>,
    pub items: Vec<TyTraitItem>,
    pub supertraits: Vec<parsed::Supertrait>,
    pub visibility: Visibility,
    pub attributes: transform::AttributesMap,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum TyTraitInterfaceItem {
    TraitFn(DeclRefTraitFn),
}

#[derive(Clone, Debug)]
pub enum TyTraitItem {
    Fn(DeclRefFunction),
}

impl Named for TyTraitDeclaration {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl Spanned for TyTraitDeclaration {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl EqWithEngines for TyTraitDeclaration {}
impl PartialEqWithEngines for TyTraitDeclaration {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.name == other.name
            && self.type_parameters.eq(&other.type_parameters, engines)
            && self.interface_surface.eq(&other.interface_surface, engines)
            && self.items.eq(&other.items, engines)
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
            items,
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
        items.hash(state, engines);
        supertraits.hash(state, engines);
        visibility.hash(state);
    }
}

impl EqWithEngines for TyTraitInterfaceItem {}
impl PartialEqWithEngines for TyTraitInterfaceItem {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        match (self, other) {
            (TyTraitInterfaceItem::TraitFn(id), TyTraitInterfaceItem::TraitFn(other_id)) => {
                id.eq(other_id, engines)
            }
        }
    }
}

impl EqWithEngines for TyTraitItem {}
impl PartialEqWithEngines for TyTraitItem {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        match (self, other) {
            (TyTraitItem::Fn(id), TyTraitItem::Fn(other_id)) => id.eq(other_id, engines),
        }
    }
}

impl HashWithEngines for TyTraitInterfaceItem {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        match self {
            TyTraitInterfaceItem::TraitFn(fn_decl) => fn_decl.hash(state, engines),
        }
    }
}

impl HashWithEngines for TyTraitItem {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        match self {
            TyTraitItem::Fn(fn_decl) => fn_decl.hash(state, engines),
        }
    }
}

impl SubstTypes for TyTraitDeclaration {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
        self.interface_surface
            .iter_mut()
            .for_each(|item| match item {
                TyTraitInterfaceItem::TraitFn(function_decl_ref) => {
                    let new_decl_ref = function_decl_ref
                        .clone()
                        .subst_types_and_insert_new(type_mapping, engines);
                    function_decl_ref.replace_id((&new_decl_ref).into());
                }
            });
        // we don't have to type check the methods because it hasn't been type checked yet
    }
}

impl SubstTypes for TyTraitItem {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        match self {
            TyTraitItem::Fn(fn_decl) => fn_decl.subst(type_mapping, engines),
        }
    }
}

impl ReplaceSelfType for TyTraitDeclaration {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.replace_self_type(engines, self_type));
        self.interface_surface
            .iter_mut()
            .for_each(|item| match item {
                TyTraitInterfaceItem::TraitFn(function_decl_ref) => {
                    let new_decl_ref = function_decl_ref
                        .clone()
                        .replace_self_type_and_insert_new(engines, self_type);
                    function_decl_ref.replace_id((&new_decl_ref).into());
                }
            });
        // we don't have to type check the methods because it hasn't been type checked yet
    }
}

impl ReplaceSelfType for TyTraitInterfaceItem {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        match self {
            TyTraitInterfaceItem::TraitFn(fn_decl) => fn_decl.replace_self_type(engines, self_type),
        }
    }
}

impl ReplaceSelfType for TyTraitItem {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        match self {
            TyTraitItem::Fn(fn_decl) => fn_decl.replace_self_type(engines, self_type),
        }
    }
}

impl ReplaceFunctionImplementingType for TyTraitItem {
    fn replace_implementing_type(
        &mut self,
        engines: Engines<'_>,
        implementing_type: TyDeclaration,
    ) {
        match self {
            TyTraitItem::Fn(decl_ref) => {
                decl_ref.replace_implementing_type(engines, implementing_type)
            }
        }
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
