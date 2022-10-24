use crate::{language::parsed::Expression, type_system::TypeInfo, Ident};

use sway_types::span::Span;

#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    pub name:                 Ident,
    pub type_ascription:      TypeInfo,
    pub type_ascription_span: Option<Span>,
    pub body:                 Expression, // will be codeblock variant
    pub is_mutable:           bool,
}
