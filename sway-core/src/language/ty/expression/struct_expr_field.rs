use crate::{CopyTypes, Ident, TypeMapping};

use super::TyExpression;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TyStructExpressionField {
    pub name: Ident,
    pub value: TyExpression,
}

impl CopyTypes for TyStructExpressionField {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.value.copy_types(type_mapping);
    }
}
