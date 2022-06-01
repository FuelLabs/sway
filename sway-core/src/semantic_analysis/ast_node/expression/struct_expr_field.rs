use crate::semantic_analysis::{CopyTypes, TypeMapping, TypedExpression};
use crate::Ident;

#[derive(Clone, Debug, PartialEq)]
pub struct TypedStructExpressionField {
    pub name: Ident,
    pub value: TypedExpression,
}

impl CopyTypes for TypedStructExpressionField {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.value.copy_types(type_mapping);
    }
}
