use sway_types::{Ident, Span};

use crate::{language::ty::*, type_system::*};

#[derive(Clone, Debug, Eq)]
pub struct TyVariableDeclaration {
    pub name: Ident,
    pub body: TyExpression,
    pub mutability: VariableMutability,
    pub type_ascription: TypeId,
    pub type_ascription_span: Option<Span>,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TyVariableDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.body == other.body
            && self.mutability == other.mutability
            && look_up_type_id(self.type_ascription) == look_up_type_id(other.type_ascription)
    }
}

impl CopyTypes for TyVariableDeclaration {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, type_engine: &TypeEngine) {
        self.type_ascription.copy_types(type_mapping, type_engine);
        self.body.copy_types(type_mapping, type_engine)
    }
}

impl ReplaceSelfType for TyVariableDeclaration {
    fn replace_self_type(&mut self, type_engine: &TypeEngine, self_type: TypeId) {
        self.type_ascription
            .replace_self_type(type_engine, self_type);
        self.body.replace_self_type(type_engine, self_type)
    }
}
