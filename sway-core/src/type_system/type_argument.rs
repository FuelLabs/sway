use crate::type_system::*;
use std::{
    fmt,
    hash::{Hash, Hasher},
};
use sway_types::{Span, Spanned};

#[derive(Debug, Clone, Eq)]
pub struct TypeArgument {
    pub type_id: TypeId,
    pub initial_type_id: TypeId,
    pub span: Span,
}

impl Spanned for TypeArgument {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl Hash for TypeArgument {
    fn hash<H: Hasher>(&self, state: &mut H) {
        look_up_type_id(self.type_id).hash(state);
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypeArgument {
    fn eq(&self, other: &Self) -> bool {
        look_up_type_id(self.type_id) == look_up_type_id(other.type_id)
    }
}

impl Default for TypeArgument {
    fn default() -> Self {
        let initial_type_id = insert_type(TypeInfo::Unknown);
        TypeArgument {
            type_id: initial_type_id,
            initial_type_id,
            span: Span::dummy(),
        }
    }
}

impl fmt::Display for TypeArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", look_up_type_id(self.type_id))
    }
}

impl TypeArgument {
    pub fn json_abi_str(&self) -> String {
        look_up_type_id(self.type_id).json_abi_str()
    }
}

impl ReplaceSelfType for TypeArgument {
    fn replace_self_type(&mut self, self_type: TypeId) {
        self.type_id.replace_self_type(self_type);
    }
}

impl CopyTypes for TypeArgument {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping) {
        self.type_id.copy_types(type_mapping);
    }
}
