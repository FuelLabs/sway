use std::hash::{Hash, Hasher};

use super::{FunctionDeclaration, FunctionParameter};

use crate::{decl_engine::DeclId, engine_threading::*, language::*, transform, type_system::*};
use sway_types::{ident::Ident, span::Span, Spanned};

#[derive(Debug, Clone)]
pub struct TraitDeclaration {
    pub name: Ident,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub attributes: transform::AttributesMap,
    pub interface_surface: Vec<TraitFn>,
    pub methods: Vec<FunctionDeclaration>,
    pub supertraits: Vec<Supertrait>,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Supertrait {
    pub name: CallPath,
    pub decl_id: Option<DeclId>,
}

impl Spanned for Supertrait {
    fn span(&self) -> Span {
        self.name.span()
    }
}

impl EqWithEngines for Supertrait {}
impl PartialEqWithEngines for Supertrait {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.name == other.name && self.decl_id.eq(&other.decl_id, engines)
    }
}

impl HashWithEngines for Supertrait {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        self.name.hash(state);
        self.decl_id.hash(state, engines);
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
