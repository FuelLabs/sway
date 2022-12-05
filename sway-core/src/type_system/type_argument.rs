use crate::{engine_threading::*, type_system::*};
use std::{fmt, hash::Hasher};
use sway_types::{Span, Spanned};

#[derive(Debug, Clone)]
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
impl HashWithEngines for TypeArgument {
    fn hash<H: Hasher>(&self, state: &mut H, type_engine: &TypeEngine) {
        type_engine
            .look_up_type_id(self.type_id)
            .hash(state, type_engine);
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl EqWithEngines for TypeArgument {}
impl PartialEqWithEngines for TypeArgument {
    fn eq(&self, other: &Self, type_engine: &TypeEngine) -> bool {
        type_engine
            .look_up_type_id(self.type_id)
            .eq(&type_engine.look_up_type_id(other.type_id), type_engine)
    }
}
impl OrdWithEngines for TypeArgument {
    fn cmp(&self, rhs: &Self, _: &TypeEngine) -> std::cmp::Ordering {
        self.type_id
            .cmp(&rhs.type_id)
            .then_with(|| self.initial_type_id.cmp(&rhs.initial_type_id))
    }
}

impl DisplayWithEngines for TypeArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, type_engine: &TypeEngine) -> fmt::Result {
        write!(
            f,
            "{}",
            type_engine.help_out(type_engine.look_up_type_id(self.type_id))
        )
    }
}

impl From<&TypeParameter> for TypeArgument {
    fn from(type_param: &TypeParameter) -> Self {
        TypeArgument {
            type_id: type_param.type_id,
            initial_type_id: type_param.initial_type_id,
            span: type_param.name_ident.span(),
        }
    }
}

impl TypeArgument {
    pub fn json_abi_str(&self, type_engine: &TypeEngine) -> String {
        type_engine
            .look_up_type_id(self.type_id)
            .json_abi_str(type_engine)
    }
}

impl ReplaceSelfType for TypeArgument {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.type_id.replace_self_type(engines, self_type);
    }
}

impl CopyTypes for TypeArgument {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, engines: Engines<'_>) {
        self.type_id.copy_types(type_mapping, engines);
    }
}
