use sway_types::{Named, Spanned};

use crate::{language::parsed::Expression, Ident, TypeArgument};

#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    pub name: Ident,
    pub type_ascription: TypeArgument,
    pub body: Expression, // will be codeblock variant
    pub is_mutable: bool,
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
