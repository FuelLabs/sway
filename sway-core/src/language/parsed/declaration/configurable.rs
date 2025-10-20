use crate::{
    engine_threading::DebugWithEngines,
    language::{parsed::Expression, Visibility},
    transform, Engines, GenericTypeArgument,
};
use sway_types::{Ident, Named, Span, Spanned};

#[derive(Debug, Clone)]
pub struct ConfigurableDeclaration {
    pub name: Ident,
    pub attributes: transform::Attributes,
    pub type_ascription: GenericTypeArgument,
    pub value: Option<Expression>,
    pub visibility: Visibility,
    pub span: Span,
    pub block_keyword_span: Span,
}

impl Named for ConfigurableDeclaration {
    fn name(&self) -> &sway_types::BaseIdent {
        &self.name
    }
}

impl Spanned for ConfigurableDeclaration {
    fn span(&self) -> sway_types::Span {
        self.span.clone()
    }
}

impl DebugWithEngines for ConfigurableDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, _engines: &Engines) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.name))
    }
}
