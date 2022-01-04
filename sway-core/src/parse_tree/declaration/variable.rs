use crate::parse_tree::Expression;
use crate::Span;
use crate::{type_engine::TypeInfo, Ident};

#[derive(Debug, Clone)]
pub struct VariableDeclaration<'sc> {
    pub name: Ident<'sc>,
    pub type_ascription: TypeInfo,
    pub type_ascription_span: Option<Span<'sc>>,
    pub body: Expression<'sc>, // will be codeblock variant
    pub is_mutable: bool,
}
