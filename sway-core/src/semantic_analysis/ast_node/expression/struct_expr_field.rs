use crate::declaration_engine::declaration_engine::DeclarationEngine;
use crate::types::{CompileWrapper, ToCompileWrapper};
use crate::Ident;
use crate::{semantic_analysis::*, type_system::*};

#[derive(Clone, Debug)]
pub struct TypedStructExpressionField {
    pub name: Ident,
    pub value: TypedExpression,
}

impl PartialEq for CompileWrapper<'_, TypedStructExpressionField> {
    fn eq(&self, other: &Self) -> bool {
        let CompileWrapper {
            inner: me,
            declaration_engine: de,
        } = self;
        let CompileWrapper { inner: them, .. } = other;
        me.name == them.name && me.value.wrap_ref(de) == them.value.wrap_ref(de)
    }
}

impl CopyTypes for TypedStructExpressionField {
    fn copy_types(&mut self, type_mapping: &TypeMapping, de: &DeclarationEngine) {
        self.value.copy_types(type_mapping, de);
    }
}
