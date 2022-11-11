use sway_types::Ident;

use crate::{language::ty::*, type_system::*};

#[derive(Clone, Debug)]
pub struct TyAsmRegisterDeclaration {
    pub(crate) initializer: Option<TyExpression>,
    pub(crate) name: Ident,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TyAsmRegisterDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && if let (Some(l), Some(r)) = (self.initializer.clone(), other.initializer.clone()) {
                l == r
            } else {
                true
            }
    }
}

impl CopyTypes for TyAsmRegisterDeclaration {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, type_engine: &TypeEngine) {
        if let Some(ref mut initializer) = self.initializer {
            initializer.copy_types(type_mapping, type_engine)
        }
    }
}

impl ReplaceSelfType for TyAsmRegisterDeclaration {
    fn replace_self_type(&mut self, type_engine: &TypeEngine, self_type: TypeId) {
        if let Some(ref mut initializer) = self.initializer {
            initializer.replace_self_type(type_engine, self_type)
        }
    }
}
