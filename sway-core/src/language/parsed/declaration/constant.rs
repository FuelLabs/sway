use crate::{
    engine_threading::{
        DebugWithEngines, EqWithEngines, PartialEqWithEngines, PartialEqWithEnginesContext,
    },
    language::{parsed::Expression, Visibility},
    transform, Engines, GenericTypeArgument,
};
use sway_types::{Ident, Named, Span, Spanned};

#[derive(Debug, Clone)]
pub struct ConstantDeclaration {
    pub name: Ident,
    pub attributes: transform::Attributes,
    pub type_ascription: GenericTypeArgument,
    pub value: Option<Expression>,
    pub visibility: Visibility,
    pub span: Span,
}

impl EqWithEngines for ConstantDeclaration {}
impl PartialEqWithEngines for ConstantDeclaration {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name == other.name
            && self.attributes == other.attributes
            && self.type_ascription.eq(&other.type_ascription, ctx)
            && self.value.eq(&other.value, ctx)
            && self.visibility == other.visibility
            && self.span == other.span
    }
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
