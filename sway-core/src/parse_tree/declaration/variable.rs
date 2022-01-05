use crate::parse_tree::Expression;
use crate::Span;
use crate::{type_engine::TypeInfo, Ident};

#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    pub name: Ident,
    pub type_ascription: TypeInfo,
    pub type_ascription_span: Option<Span>,
    pub body: Expression, // will be codeblock variant
    pub is_mutable: bool,
}
