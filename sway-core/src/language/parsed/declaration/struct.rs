use crate::{
    ast_elements::type_argument::GenericTypeArgument, engine_threading::{EqWithEngines, PartialEqWithEngines, PartialEqWithEnginesContext}, language::Visibility, transform, type_system::TypeParameter, GenericArgument
};
use sway_types::{ident::Ident, span::Span, Named, Spanned};

#[derive(Debug, Clone)]
pub struct StructDeclaration {
    pub name: Ident,
    pub attributes: transform::Attributes,
    pub fields: Vec<StructField>,
    pub type_parameters: Vec<TypeParameter>,
    pub visibility: Visibility,
    pub(crate) span: Span,
}

impl EqWithEngines for StructDeclaration {}
impl PartialEqWithEngines for StructDeclaration {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name == other.name
            && self.type_parameters.eq(&other.type_parameters, ctx)
            && self.attributes == other.attributes
            && self.fields.eq(&other.fields, ctx)
            && self.type_parameters.eq(&other.type_parameters, ctx)
            && self.visibility == other.visibility
            && self.span == other.span
    }
}

impl Named for StructDeclaration {
    fn name(&self) -> &sway_types::BaseIdent {
        &self.name
    }
}

impl Spanned for StructDeclaration {
    fn span(&self) -> sway_types::Span {
        self.span.clone()
    }
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub visibility: Visibility,
    pub name: Ident,
    pub attributes: transform::Attributes,
    pub(crate) span: Span,
    pub type_argument: GenericTypeArgument,
}

impl EqWithEngines for StructField {}
impl PartialEqWithEngines for StructField {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.visibility == other.visibility
            && self.name == other.name
            && self.attributes == other.attributes
            && self.span == other.span
            && self.type_argument.eq(&other.type_argument, ctx)
    }
}
