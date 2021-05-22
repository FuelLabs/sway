use crate::parse_tree::Expression;
use crate::{types::TypeInfo, Ident};

#[derive(Debug, Clone)]
pub struct VariableDeclaration<'sc> {
    pub name: Ident<'sc>,
    pub type_ascription: Option<TypeInfo<'sc>>,
    pub body: Expression<'sc>, // will be codeblock variant
    pub is_mutable: bool,
}
