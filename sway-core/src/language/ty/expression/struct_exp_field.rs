use sway_types::Ident;

use crate::{language::ty::*, type_system::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TyStructExpressionField {
    pub name: Ident,
    pub value: TyExpression,
}

impl CopyTypes for TyStructExpressionField {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping) {
        self.value.copy_types(type_mapping);
    }
}
