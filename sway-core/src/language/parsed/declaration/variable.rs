use crate::{language::parsed::Expression, Ident, TypeArgument};

#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    pub name: Ident,
    pub type_ascription: TypeArgument,
    pub body: Expression, // will be codeblock variant
    pub is_mutable: bool,
}

impl deepsize::DeepSizeOf for VariableDeclaration {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        0
    }
}
