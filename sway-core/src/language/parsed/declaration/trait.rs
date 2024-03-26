use std::hash::{Hash, Hasher};

use super::{ConstantDeclaration, FunctionDeclaration, FunctionParameter};

use crate::{
    decl_engine::{parsed_id::ParsedDeclId, DeclRefTrait},
    engine_threading::*,
    language::*,
    transform,
    type_system::*,
};
use sway_error::handler::ErrorEmitted;
use sway_types::{ident::Ident, span::Span, Spanned};

#[derive(Debug, Clone)]
pub enum TraitItem {
    TraitFn(ParsedDeclId<TraitFn>),
    Constant(ParsedDeclId<ConstantDeclaration>),
    Type(ParsedDeclId<TraitTypeDeclaration>),
    // to handle parser recovery: Error represents an incomplete trait item
    Error(Box<[Span]>, ErrorEmitted),
}

impl deepsize::DeepSizeOf for TraitItem {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        0
    }
}

#[derive(Debug, Clone)]
pub struct TraitDeclaration {
    pub name: Ident,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub attributes: transform::AttributesMap,
    pub interface_surface: Vec<TraitItem>,
    pub methods: Vec<ParsedDeclId<FunctionDeclaration>>,
    pub supertraits: Vec<Supertrait>,
    pub visibility: Visibility,
    pub span: Span,
}

impl deepsize::DeepSizeOf for TraitDeclaration {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        0
    }
}

#[derive(Debug, Clone, deepsize::DeepSizeOf)]
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
    fn eq(&self, other: &Self, engines: &Engines) -> bool {
        let Supertrait {
            name: ln,
            decl_ref: ldr,
        } = self;
        let Supertrait {
            name: rn,
            decl_ref: rdr,
        } = other;
        ln == rn && ldr.eq(rdr, engines)
    }
}

impl HashWithEngines for Supertrait {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let Supertrait { name, decl_ref } = self;
        name.hash(state);
        decl_ref.hash(state, engines);
    }
}

#[derive(Debug, Clone, deepsize::DeepSizeOf)]
pub struct TraitFn {
    pub name: Ident,
    pub span: Span,
    pub attributes: transform::AttributesMap,
    pub purity: Purity,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: TypeArgument,
}

#[derive(Debug, Clone)]
pub struct TraitTypeDeclaration {
    pub name: Ident,
    pub attributes: transform::AttributesMap,
    pub ty_opt: Option<TypeArgument>,
    pub span: Span,
}

impl deepsize::DeepSizeOf for TraitTypeDeclaration {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        0
    }
}

impl DebugWithEngines for TraitTypeDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, _engines: &Engines) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.name))
    }
}
