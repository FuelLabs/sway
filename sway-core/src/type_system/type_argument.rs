use crate::{type_system::*, types::*};
use std::{
    fmt,
    hash::{Hash, Hasher},
};
use sway_types::{Property, Span};

#[derive(Debug, Clone)]
pub struct TypeArgument {
    pub type_id: TypeId,
    pub initial_type_id: TypeId,
    pub span: Span,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for CompileWrapper<'_, TypeArgument> {
    fn eq(&self, other: &Self) -> bool {
        let CompileWrapper {
            inner: me,
            declaration_engine: de,
        } = self;
        let CompileWrapper { inner: them, .. } = other;
        look_up_type_id(me.type_id).wrap(de) == look_up_type_id(them.type_id).wrap(de)
    }
}

impl PartialEq for CompileWrapper<'_, Vec<TypeArgument>> {
    fn eq(&self, other: &Self) -> bool {
        let CompileWrapper {
            inner: me,
            declaration_engine: de,
        } = self;
        let CompileWrapper { inner: them, .. } = other;
        if me.len() != them.len() {
            return false;
        }
        me.iter()
            .map(|elem| elem.wrap(de))
            .zip(other.inner.iter().map(|elem| elem.wrap(de)))
            .map(|(left, right)| left == right)
            .all(|elem| elem)
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl Hash for CompileWrapper<'_, TypeArgument> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let CompileWrapper {
            inner: me,
            declaration_engine: de,
        } = self;
        look_up_type_id(me.type_id).wrap(de).hash(state);
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

impl JsonAbiString for TypeArgument {
    fn json_abi_str(&self) -> String {
        look_up_type_id(self.type_id).json_abi_str()
    }
}

impl ToJsonAbi for TypeArgument {
    type Output = Property;

    fn generate_json_abi(&self) -> Self::Output {
        Property {
            name: "__tuple_element".to_string(),
            type_field: self.type_id.json_abi_str(),
            components: self.type_id.generate_json_abi(),
            type_arguments: self
                .type_id
                .get_type_parameters()
                .map(|v| v.iter().map(TypeParameter::generate_json_abi).collect()),
        }
    }
}

impl ReplaceSelfType for TypeArgument {
    fn replace_self_type(&mut self, self_type: TypeId) {
        self.type_id.replace_self_type(self_type);
    }
}
