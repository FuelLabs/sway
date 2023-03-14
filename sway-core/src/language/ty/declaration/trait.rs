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

impl SubstTypes for TyTraitDeclaration {
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
            });
        self.items.iter_mut().for_each(|item| match item {
            TyTraitItem::Fn(item_ref) => {
                item_ref.subst(type_mapping, engines);
            }
        });
    }
}

impl SubstTypes for TyTraitItem {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        match self {
            TyTraitItem::Fn(fn_decl) => fn_decl.subst(type_mapping, engines),
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
