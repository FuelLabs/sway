use sway_types::{Named, Spanned};

use crate::{
    engine_threading::{EqWithEngines, PartialEqWithEngines, PartialEqWithEnginesContext},
    language::parsed::Expression,
    Ident, TypeArgument,
};

#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    pub name: Ident,
    pub type_ascription: TypeArgument,
    pub body: Expression, // will be codeblock variant
    pub is_mutable: bool,
}

impl EqWithEngines for VariableDeclaration {}
impl PartialEqWithEngines for VariableDeclaration {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name == other.name
            && self.type_ascription.eq(&other.type_ascription, ctx)
            && self.body.eq(&other.body, ctx)
            && self.is_mutable == other.is_mutable
    }
}

impl Named for VariableDeclaration {
    fn name(&self) -> &sway_types::BaseIdent {
        &self.name
    }
}

impl Spanned for VariableDeclaration {
    fn span(&self) -> sway_types::Span {
        self.name.span()
    }
}
