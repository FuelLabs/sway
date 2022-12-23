use sway_types::{Ident, Span};

use crate::{engine_threading::*, language::ty::*, type_system::*};

#[derive(Clone, Debug)]
pub struct TyVariableDeclaration {
    pub name: Ident,
    pub body: TyExpression,
    pub mutability: VariableMutability,
    pub return_type: TypeId,
    pub type_ascription: TypeId,
    pub type_ascription_span: Option<Span>,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl EqWithEngines for TyVariableDeclaration {}
impl PartialEqWithEngines for TyVariableDeclaration {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let type_engine = engines.te();
        self.name == other.name
            && self.body.eq(&other.body, engines)
            && self.mutability == other.mutability
            && type_engine
                .look_up_type_id(self.return_type)
                .eq(&type_engine.look_up_type_id(other.return_type), engines)
            && type_engine
                .look_up_type_id(self.type_ascription)
                .eq(&type_engine.look_up_type_id(other.type_ascription), engines)
    }
}

impl CopyTypes for TyVariableDeclaration {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, engines: Engines<'_>) {
        self.return_type.copy_types(type_mapping, engines);
        self.type_ascription.copy_types(type_mapping, engines);
        self.body.copy_types(type_mapping, engines)
    }
}

impl ReplaceSelfType for TyVariableDeclaration {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.return_type.replace_self_type(engines, self_type);
        self.type_ascription.replace_self_type(engines, self_type);
        self.body.replace_self_type(engines, self_type)
    }
}
