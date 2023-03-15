use std::hash::{Hash, Hasher};

use sway_types::Ident;

use crate::{engine_threading::*, language::ty::*};

#[derive(Clone, Debug)]
pub struct TyAsmRegisterDeclaration {
    pub initializer: Option<TyExpression>,
    pub(crate) name: Ident,
}

impl PartialEqWithEngines for TyAsmRegisterDeclaration {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.name == other.name
            && if let (Some(l), Some(r)) = (&self.initializer, &other.initializer) {
                l.eq(r, engines)
            } else {
                true
            }
    }
}

impl HashWithEngines for TyAsmRegisterDeclaration {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TyAsmRegisterDeclaration { initializer, name } = self;
        name.hash(state);
        if let Some(x) = initializer.as_ref() {
            x.hash(state, engines)
        }
    }
}
