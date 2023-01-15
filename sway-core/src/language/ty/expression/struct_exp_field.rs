use sway_types::Ident;

use crate::{decl_engine::*, engine_threading::*, language::ty::*, type_system::*};

#[derive(Clone, Debug)]
pub struct TyStructExpressionField {
    pub name: Ident,
    pub value: TyExpression,
}

impl EqWithEngines for TyStructExpressionField {}
impl PartialEqWithEngines for TyStructExpressionField {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.name == other.name && self.value.eq(&other.value, engines)
    }
}

impl SubstTypes for TyStructExpressionField {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.value.subst(type_mapping, engines);
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
