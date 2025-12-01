use crate::{engine_threading::*, language::ty::*};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use sway_types::Ident;

#[derive(Clone, Debug, Serialize, Deserialize)]
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

// impl SubstTypes for TyAsmRegisterDeclaration {
//     fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
//         self.initializer.subst(ctx)
//     }
// }
