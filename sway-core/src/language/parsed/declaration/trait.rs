use std::hash::{Hash, Hasher};

use super::{FunctionDeclaration, FunctionParameter};

use crate::{
    decl_engine::DeclRefTrait, engine_threading::*, language::*, transform, type_system::*,
};
use sway_types::{ident::Ident, span::Span, Spanned};

#[derive(Debug, Clone)]
pub enum TraitItem {
    TraitFn(TraitFn),
}

#[derive(Debug, Clone)]
pub struct TraitDeclaration {
    pub name: Ident,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub attributes: transform::AttributesMap,
    pub interface_surface: Vec<TraitItem>,
    pub methods: Vec<FunctionDeclaration>,
    pub supertraits: Vec<Supertrait>,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Supertrait {
    pub name: CallPath,
    pub decl_ref: Option<DeclRefTrait>,
}

impl Spanned for Supertrait {
    fn span(&self) -> Span {
        self.name.span()
    }
}

impl EqWithEngines for Supertrait {}
impl PartialEqWithEngines for Supertrait {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.name == other.name && self.decl_ref.eq(&other.decl_ref, engines)
    }
}

impl HashWithEngines for Supertrait {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let Supertrait { name, decl_ref } = self;
        name.hash(state);
        decl_ref.hash(state, engines);
    }
}

#[derive(Debug, Clone)]
pub struct TraitFn {
    pub name: Ident,
    pub attributes: transform::AttributesMap,
    pub purity: Purity,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: TypeInfo,
    pub return_type_span: Span,
}
