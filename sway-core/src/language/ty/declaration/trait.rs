use std::hash::{Hash, Hasher};

use sway_types::{Ident, Named, Span, Spanned};

use crate::{
    decl_engine::{
        DeclRefConstant, DeclRefFunction, DeclRefTraitFn, ReplaceFunctionImplementingType,
    },
    engine_threading::*,
    language::{parsed, Visibility},
    transform,
    type_system::*,
};

use super::TyDecl;

#[derive(Clone, Debug)]
pub struct TyTraitDecl {
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
    Constant(DeclRefConstant),
}

#[derive(Clone, Debug)]
pub enum TyTraitItem {
    Fn(DeclRefFunction),
    Constant(DeclRefConstant),
}

impl Named for TyTraitDecl {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl Spanned for TyTraitDecl {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl EqWithEngines for TyTraitDecl {}
impl PartialEqWithEngines for TyTraitDecl {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.name == other.name
            && self.type_parameters.eq(&other.type_parameters, engines)
            && self.interface_surface.eq(&other.interface_surface, engines)
            && self.items.eq(&other.items, engines)
            && self.supertraits.eq(&other.supertraits, engines)
            && self.visibility == other.visibility
    }
}

impl HashWithEngines for TyTraitDecl {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TyTraitDecl {
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
            (TyTraitInterfaceItem::Constant(id), TyTraitInterfaceItem::Constant(other_id)) => {
                id.eq(other_id, engines)
            }
            _ => false,
        }
    }
}

impl EqWithEngines for TyTraitItem {}
impl PartialEqWithEngines for TyTraitItem {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        match (self, other) {
            (TyTraitItem::Fn(id), TyTraitItem::Fn(other_id)) => id.eq(other_id, engines),
            (TyTraitItem::Constant(id), TyTraitItem::Constant(other_id)) => {
                id.eq(other_id, engines)
            }
            _ => false,
        }
    }
}

impl HashWithEngines for TyTraitInterfaceItem {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        match self {
            TyTraitInterfaceItem::TraitFn(fn_decl) => fn_decl.hash(state, engines),
            TyTraitInterfaceItem::Constant(const_decl) => const_decl.hash(state, engines),
        }
    }
}

impl HashWithEngines for TyTraitItem {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        match self {
            TyTraitItem::Fn(fn_decl) => fn_decl.hash(state, engines),
            TyTraitItem::Constant(const_decl) => const_decl.hash(state, engines),
        }
    }
}

impl SubstTypes for TyTraitDecl {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
        self.interface_surface
            .iter_mut()
            .for_each(|item| match item {
                TyTraitInterfaceItem::TraitFn(item_ref) => {
                    item_ref.subst(type_mapping, engines);
                }
                TyTraitInterfaceItem::Constant(decl_ref) => {
                    decl_ref.subst(type_mapping, engines);
                }
            });
        self.items.iter_mut().for_each(|item| match item {
            TyTraitItem::Fn(item_ref) => {
                item_ref.subst(type_mapping, engines);
            }
            TyTraitItem::Constant(item_ref) => {
                item_ref.subst(type_mapping, engines);
            }
        });
    }
}

impl SubstTypes for TyTraitItem {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        match self {
            TyTraitItem::Fn(fn_decl) => fn_decl.subst(type_mapping, engines),
            TyTraitItem::Constant(const_decl) => const_decl.subst(type_mapping, engines),
        }
    }
}

impl ReplaceFunctionImplementingType for TyTraitItem {
    fn replace_implementing_type(&mut self, engines: Engines<'_>, implementing_type: TyDecl) {
        match self {
            TyTraitItem::Fn(decl_ref) => {
                decl_ref.replace_implementing_type(engines, implementing_type)
            }
            TyTraitItem::Constant(_decl_ref) => {
                // ignore, only needed for functions
            }
        }
    }
}

impl MonomorphizeHelper for TyTraitDecl {
    fn name(&self) -> &Ident {
        &self.name
    }

    fn type_parameters(&self) -> &[TypeParameter] {
        &self.type_parameters
    }
}
