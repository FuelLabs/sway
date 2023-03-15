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
