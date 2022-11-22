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
    fn eq(&self, other: &Self, type_engine: &TypeEngine) -> bool {
        self.name == other.name
            && self.body.eq(&other.body, type_engine)
            && self.mutability == other.mutability
            && type_engine
                .look_up_type_id(self.return_type)
                .eq(&type_engine.look_up_type_id(other.return_type), type_engine)
            && type_engine.look_up_type_id(self.type_ascription).eq(
                &type_engine.look_up_type_id(other.type_ascription),
                type_engine,
            )
    }
}

impl CopyTypes for TyVariableDeclaration {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, type_engine: &TypeEngine) {
        self.return_type.copy_types(type_mapping, type_engine);
        self.type_ascription.copy_types(type_mapping, type_engine);
        self.body.copy_types(type_mapping, type_engine)
    }
}

impl ReplaceSelfType for TyVariableDeclaration {
    fn replace_self_type(&mut self, type_engine: &TypeEngine, self_type: TypeId) {
        self.return_type.replace_self_type(type_engine, self_type);
        self.type_ascription
            .replace_self_type(type_engine, self_type);
        self.body.replace_self_type(type_engine, self_type)
    }
}
