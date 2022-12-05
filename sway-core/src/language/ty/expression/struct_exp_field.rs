use sway_types::Ident;

use crate::{
    declaration_engine::{DeclMapping, ReplaceDecls},
    engine_threading::*,
    language::ty::*,
    type_system::*,
};

#[derive(Clone, Debug)]
pub struct TyStructExpressionField {
    pub name: Ident,
    pub value: TyExpression,
}

impl EqWithEngines for TyStructExpressionField {}
impl PartialEqWithEngines for TyStructExpressionField {
    fn eq(&self, rhs: &Self, type_engine: &TypeEngine) -> bool {
        self.name == rhs.name && self.value.eq(&rhs.value, type_engine)
    }
}

impl CopyTypes for TyStructExpressionField {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, engines: Engines<'_>) {
        self.value.copy_types(type_mapping, engines);
    }
}

impl ReplaceSelfType for TyStructExpressionField {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.value.replace_self_type(engines, self_type);
    }
}

impl ReplaceDecls for TyStructExpressionField {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping, engines: Engines<'_>) {
        self.value.replace_decls(decl_mapping, engines);
    }
}
