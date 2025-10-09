use crate::{
    ast_elements::type_argument::GenericTypeArgument,
    engine_threading::{EqWithEngines, PartialEqWithEngines, PartialEqWithEnginesContext},
    language::Visibility,
    transform,
};

use sway_types::{ident::Ident, span::Span, Named, Spanned};

#[derive(Debug, Clone)]
pub struct TypeAliasDeclaration {
    pub name: Ident,
    pub attributes: transform::Attributes,
    pub ty: GenericTypeArgument,
    pub visibility: Visibility,
    pub span: Span,
}

impl EqWithEngines for TypeAliasDeclaration {}
impl PartialEqWithEngines for TypeAliasDeclaration {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name == other.name
            && self.attributes == other.attributes
            && self.ty.eq(&other.ty, ctx)
            && self.visibility == other.visibility
            && self.span == other.span
    }
}

impl Named for TypeAliasDeclaration {
    fn name(&self) -> &sway_types::BaseIdent {
        &self.name
    }
}

impl Spanned for TypeAliasDeclaration {
    fn span(&self) -> sway_types::Span {
        self.span.clone()
    }
}
