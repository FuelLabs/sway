use crate::{type_engine::*, Ident, TypeParameter};

use sway_types::span::Span;

#[derive(Debug, Clone, Eq)]
pub struct TypedFunctionParameter {
    pub(crate) name: Ident,
    pub(crate) r#type: TypeId,
    pub(crate) type_span: Span,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypedFunctionParameter {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && look_up_type_id(self.r#type) == look_up_type_id(other.r#type)
    }
}

impl TypedFunctionParameter {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.r#type = match look_up_type_id(self.r#type).matches_type_parameter(type_mapping) {
            Some(matching_id) => insert_type(TypeInfo::Ref(matching_id)),
            None => insert_type(look_up_type_id_raw(self.r#type)),
        };
    }
}
