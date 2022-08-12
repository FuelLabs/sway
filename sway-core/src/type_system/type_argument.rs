use crate::{type_system::*, types::*};
use std::{
    fmt,
    hash::{Hash, Hasher},
};
use sway_types::{Property, Span};

#[derive(Debug, Clone)]
pub struct TypeArgument {
    pub type_id: TypeId,
    pub span: Span,
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

// impl Default for TypeArgument {
//     fn default() -> Self {
//         TypeArgument {
//             type_id: insert_type(TypeInfo::Unknown),
//             span: Span::dummy(),
//         }
//     }
// }

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
