use crate::semantic_analysis::TypedExpression;
use crate::Ident;
use crate::{type_engine::TypeId, TypeParameter};

#[derive(Clone, Debug)]
pub struct TypedVariableDeclaration<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) body: TypedExpression<'sc>, // will be codeblock variant
    pub(crate) is_mutable: bool,
}

impl TypedVariableDeclaration<'_> {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.body.copy_types(type_mapping)
    }
}
