use crate::semantic_analysis::TypedExpression;
use crate::Ident;
use crate::{type_engine::TypeId, TypeParameter};

#[derive(Clone, Debug)]
pub(crate) struct TypedStructExpressionField<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) value: TypedExpression<'sc>,
}

impl TypedStructExpressionField<'_> {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.value.copy_types(type_mapping);
    }
}
