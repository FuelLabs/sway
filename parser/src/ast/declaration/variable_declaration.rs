use crate::ast::{Expression, TypeInfo};

#[derive(Debug, Clone)]
pub(crate) struct VariableDeclaration<'sc> {
    pub(crate) name: &'sc str,
    pub(crate) type_ascription: Option<TypeInfo<'sc>>,
    pub(crate) body: Expression<'sc>, // will be codeblock variant
}
