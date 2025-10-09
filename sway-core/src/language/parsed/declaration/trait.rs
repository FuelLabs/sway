use super::{ConstantDeclaration, FunctionDeclaration, FunctionParameter};
use crate::{
    ast_elements::type_argument::GenericTypeArgument,
    decl_engine::{parsed_id::ParsedDeclId, DeclRefTrait},
    engine_threading::*,
    language::*,
    transform,
    type_system::*,
};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use sway_error::handler::ErrorEmitted;
use sway_types::{ident::Ident, span::Span, Named, Spanned};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraitItem {
    TraitFn(ParsedDeclId<TraitFn>),
    Constant(ParsedDeclId<ConstantDeclaration>),
    Type(ParsedDeclId<TraitTypeDeclaration>),
    // to handle parser recovery: Error represents an incomplete trait item
    Error(Box<[Span]>, #[serde(skip)] ErrorEmitted),
}

impl EqWithEngines for TraitItem {}
impl PartialEqWithEngines for TraitItem {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        match (self, other) {
            (TraitItem::TraitFn(lhs), TraitItem::TraitFn(rhs)) => {
                PartialEqWithEngines::eq(lhs, rhs, ctx)
            }
            (TraitItem::Constant(lhs), TraitItem::Constant(rhs)) => {
                PartialEqWithEngines::eq(lhs, rhs, ctx)
            }
            (TraitItem::Type(lhs), TraitItem::Type(rhs)) => PartialEqWithEngines::eq(lhs, rhs, ctx),
            (TraitItem::Error(lhs, _), TraitItem::Error(rhs, _)) => lhs.eq(rhs),
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TraitDeclaration {
    pub name: Ident,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub attributes: transform::Attributes,
    pub interface_surface: Vec<TraitItem>,
    pub methods: Vec<ParsedDeclId<FunctionDeclaration>>,
    pub supertraits: Vec<Supertrait>,
    pub visibility: Visibility,
    pub span: Span,
}

impl EqWithEngines for TraitDeclaration {}
impl PartialEqWithEngines for TraitDeclaration {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name.eq(&other.name)
            && self.type_parameters.eq(&other.type_parameters, ctx)
            && self.attributes.eq(&other.attributes)
            && self.interface_surface.eq(&other.interface_surface, ctx)
            && PartialEqWithEngines::eq(&self.methods, &other.methods, ctx)
            && self.supertraits.eq(&other.supertraits, ctx)
            && self.visibility.eq(&other.visibility)
    }
}

impl Named for TraitDeclaration {
    fn name(&self) -> &sway_types::BaseIdent {
        &self.name
    }
}

impl Spanned for TraitDeclaration {
    fn span(&self) -> sway_types::Span {
        self.span.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let Supertrait {
            name: ln,
            decl_ref: ldr,
        } = self;
        let Supertrait {
            name: rn,
            decl_ref: rdr,
        } = other;
        ln == rn && ldr.eq(rdr, ctx)
    }
}

impl HashWithEngines for Supertrait {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let Supertrait { name, decl_ref } = self;
        name.hash(state);
        decl_ref.hash(state, engines);
    }
}

#[derive(Debug, Clone)]
pub struct TraitFn {
    pub name: Ident,
    pub span: Span,
    pub attributes: transform::Attributes,
    pub purity: Purity,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: GenericTypeArgument,
}

impl Spanned for TraitFn {
    fn span(&self) -> sway_types::Span {
        self.span.clone()
    }
}

#[derive(Debug, Clone)]
pub struct TraitTypeDeclaration {
    pub name: Ident,
    pub attributes: transform::Attributes,
    pub ty_opt: Option<GenericArgument>,
    pub span: Span,
}

impl EqWithEngines for TraitTypeDeclaration {}
impl PartialEqWithEngines for TraitTypeDeclaration {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name == other.name
            && self.attributes == other.attributes
            && self.ty_opt.eq(&other.ty_opt, ctx)
    }
}

impl Named for TraitTypeDeclaration {
    fn name(&self) -> &sway_types::BaseIdent {
        &self.name
    }
}

impl Spanned for TraitTypeDeclaration {
    fn span(&self) -> sway_types::Span {
        self.span.clone()
    }
}

impl DebugWithEngines for TraitTypeDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, _engines: &Engines) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.name))
    }
}
