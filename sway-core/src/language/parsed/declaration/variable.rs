use std::fmt;

use crate::{
    engine_threading::DebugWithEngines, language::parsed::Expression, Engines, Ident, TypeArgument,
};

#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    pub name: Ident,
    pub type_ascription: TypeArgument,
    pub body: Expression, // will be codeblock variant
    pub is_mutable: bool,
}

impl DebugWithEngines for VariableDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, _engines: &Engines) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
