use crate::{type_engine::*, types::*};
use fuels_types::Property;
use std::hash::{Hash, Hasher};
use sway_types::Span;

#[derive(Debug, Clone)]
pub struct TypeArgument {
    pub(crate) type_id: TypeId,
    pub(crate) span: Span,
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
        TypeArgument {
            type_id: insert_type(TypeInfo::Unknown),
            span: Span::dummy(),
        }
    }
}

impl FriendlyTypeString for TypeArgument {
    fn friendly_type_str(&self) -> String {
        look_up_type_id(self.type_id).friendly_type_str()
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
        }
    }
}
