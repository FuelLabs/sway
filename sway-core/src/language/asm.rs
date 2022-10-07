use std::hash::{Hash, Hasher};

use sway_types::{Ident, Span};

#[derive(Debug, Clone)]
pub struct AsmOp {
    pub(crate) op_name: Ident,
    pub(crate) op_args: Vec<Ident>,
    pub(crate) span: Span,
    pub(crate) immediate: Option<Ident>,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl Hash for AsmOp {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.op_name.hash(state);
        self.op_args.hash(state);
        if let Some(immediate) = self.immediate.clone() {
            immediate.hash(state);
        }
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for AsmOp {
    fn eq(&self, other: &Self) -> bool {
        self.op_name == other.op_name
            && self.op_args == other.op_args
            && if let (Some(l), Some(r)) = (self.immediate.clone(), other.immediate.clone()) {
                l == r
            } else {
                true
            }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AsmRegister {
    pub(crate) name: String,
}

impl From<AsmRegister> for String {
    fn from(register: AsmRegister) -> String {
        register.name
    }
}
