use sway_types::Ident;

use crate::{engine_threading::*, language::ty::*, type_system::*};

#[derive(Clone, Debug)]
pub struct TyAsmRegisterDeclaration {
    pub(crate) initializer: Option<TyExpression>,
    pub(crate) name: Ident,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
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

impl SubstTypes for TyAsmRegisterDeclaration {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        if let Some(ref mut initializer) = self.initializer {
            initializer.subst(type_mapping, engines)
        }
    }
}

impl ReplaceSelfType for TyAsmRegisterDeclaration {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        if let Some(ref mut initializer) = self.initializer {
            initializer.replace_self_type(engines, self_type)
        }
    }
}
