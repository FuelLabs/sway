use crate::Ident;
use crate::{semantic_analysis::*, type_system::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedStructExpressionField {
    pub name: Ident,
    pub value: TypedExpression,
}

impl CopyTypes for TypedStructExpressionField {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.value.copy_types(type_mapping);
    }
}
