use crate::{
    ast_elements::type_argument::GenericTypeArgument, engine_threading::{EqWithEngines, PartialEqWithEngines, PartialEqWithEnginesContext}, language::Visibility, transform, type_system::*
};
use sway_types::{ident::Ident, span::Span, Named, Spanned};

#[derive(Debug, Clone)]
pub struct EnumDeclaration {
    pub name: Ident,
    pub attributes: transform::Attributes,
    pub type_parameters: Vec<TypeParameter>,
    pub variants: Vec<EnumVariant>,
    pub(crate) span: Span,
    pub visibility: Visibility,
}

impl EqWithEngines for EnumDeclaration {}
impl PartialEqWithEngines for EnumDeclaration {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name == other.name
            && self.attributes == other.attributes
            && self.type_parameters.eq(&other.type_parameters, ctx)
            && self.variants.eq(&other.variants, ctx)
            && self.visibility == other.visibility
            && self.span == other.span
    }
}

impl Named for EnumDeclaration {
    fn name(&self) -> &sway_types::BaseIdent {
        &self.name
    }
}

impl Spanned for EnumDeclaration {
    fn span(&self) -> sway_types::Span {
        self.span.clone()
    }
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: Ident,
    pub attributes: transform::Attributes,
    pub type_argument: GenericTypeArgument,
    pub(crate) tag: usize,
    pub(crate) span: Span,
}

impl EqWithEngines for EnumVariant {}
impl PartialEqWithEngines for EnumVariant {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name == other.name
            && self.attributes == other.attributes
            && self.type_argument.eq(&other.type_argument, ctx)
            && self.tag == other.tag
            && self.span == other.span
    }
}
