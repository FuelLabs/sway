use crate::parse_tree::{Expression, VarName};
use crate::types::TypeInfo;

#[derive(Debug, Clone)]
pub(crate) struct VariableDeclaration<'sc> {
    pub(crate) name: VarName<'sc>,
    pub(crate) type_ascription: Option<TypeInfo<'sc>>,
    pub(crate) body: Expression<'sc>, // will be codeblock variant
    pub(crate) is_mutable: bool,
}
