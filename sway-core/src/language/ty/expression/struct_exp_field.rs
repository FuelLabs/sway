use sway_types::Ident;

use crate::{language::ty::*, type_system::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TyStructExpressionField {
    pub name: Ident,
    pub value: TyExpression,
}

impl CopyTypes for TyStructExpressionField {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        if type_mapping.is_empty() {
            return;
        }
        self.value.copy_types(type_mapping);
    }
}
