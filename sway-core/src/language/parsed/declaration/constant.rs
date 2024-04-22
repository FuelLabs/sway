use crate::{
    engine_threading::DebugWithEngines,
    language::{parsed::Expression, Visibility},
    transform, Engines, TypeArgument,
};
use sway_types::{Ident, Named, Span, Spanned};

#[derive(Debug, Clone)]
pub struct ConstantDeclaration {
    pub name: Ident,
    pub attributes: transform::AttributesMap,
    pub type_ascription: TypeArgument,
    pub value: Option<Expression>,
    pub visibility: Visibility,
    pub is_configurable: bool,
    pub span: Span,
}

impl Named for ConstantDeclaration {
    fn name(&self) -> &sway_types::BaseIdent {
        &self.name
    }
}

impl Spanned for ConstantDeclaration {
    fn span(&self) -> sway_types::Span {
        self.span.clone()
    }
}

impl DebugWithEngines for ConstantDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, _engines: &Engines) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.name))
    }
}
