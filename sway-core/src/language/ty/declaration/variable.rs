use std::hash::{Hash, Hasher};

use sway_types::Ident;

use crate::{engine_threading::*, language::ty::*, type_system::*};

#[derive(Clone, Debug)]
pub struct TyVariableDeclaration {
    pub name: Ident,
    pub body: TyExpression,
    pub mutability: VariableMutability,
    pub return_type: TypeId,
    pub type_ascription: TypeArgument,
}

impl EqWithEngines for TyVariableDeclaration {}
impl PartialEqWithEngines for TyVariableDeclaration {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let type_engine = engines.te();
        self.name == other.name
            && self.body.eq(&other.body, engines)
            && self.mutability == other.mutability
            && type_engine
                .get(self.return_type)
                .eq(&type_engine.get(other.return_type), engines)
            && self.type_ascription.eq(&other.type_ascription, engines)
    }
}

impl HashWithEngines for TyVariableDeclaration {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TyVariableDeclaration {
            name,
            body,
            mutability,
            return_type,
            type_ascription,
        } = self;
        let type_engine = engines.te();
        name.hash(state);
        body.hash(state, engines);
        type_engine.get(*return_type).hash(state, engines);
        type_ascription.hash(state, engines);
        mutability.hash(state);
    }
}
