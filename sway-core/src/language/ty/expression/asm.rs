use std::hash::{Hash, Hasher};

use sway_types::Ident;

use crate::{engine_threading::*, language::ty::*, type_system::*};

#[derive(Clone, Debug)]
pub struct TyAsmRegisterDeclaration {
    pub initializer: Option<TyExpression>,
    pub(crate) name: Ident,
}

impl PartialEqWithEngines for TyAsmRegisterDeclaration {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name == other.name
            && if let (Some(l), Some(r)) = (&self.initializer, &other.initializer) {
                l.eq(r, ctx)
            } else {
                true
            }
    }
}

impl HashWithEngines for TyAsmRegisterDeclaration {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyAsmRegisterDeclaration { initializer, name } = self;
        name.hash(state);
        if let Some(x) = initializer.as_ref() {
            x.hash(state, engines)
        }
    }
}

impl SubstTypes for TyAsmRegisterDeclaration {
    fn subst_inner(&self, type_mapping: &TypeSubstMap, engines: &Engines) -> Option<Self> {
        let initializer = self
            .initializer
            .as_ref()
            .map(|x| x.subst(type_mapping, engines))?;
        Some(Self {
            initializer,
            name: self.name.clone(),
        })
    }
}
