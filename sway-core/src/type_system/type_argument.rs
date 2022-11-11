use crate::type_system::*;
use derivative::Derivative;
use std::{
    fmt,
    hash::{Hash, Hasher},
};
use sway_types::{Span, Spanned};

#[derive(Debug, Clone, Eq, Derivative)]
#[derivative(PartialOrd, Ord)]
pub struct TypeArgument {
    pub type_id: TypeId,
    pub initial_type_id: TypeId,
    #[derivative(PartialOrd = "ignore", Ord = "ignore")]
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

impl DisplayWithTypeEngine for TypeArgument {
    fn fmt_with_type_engine(
        &self,
        f: &mut fmt::Formatter<'_>,
        type_engine: &TypeEngine,
    ) -> fmt::Result {
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
    fn replace_self_type(&mut self, type_engine: &TypeEngine, self_type: TypeId) {
        self.type_id.replace_self_type(type_engine, self_type);
    }
}

impl CopyTypes for TypeArgument {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, type_engine: &TypeEngine) {
        self.type_id.copy_types(type_mapping, type_engine);
    }
}
