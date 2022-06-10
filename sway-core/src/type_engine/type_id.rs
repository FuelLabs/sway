use std::fmt;
use sway_types::{Span, Spanned};

use crate::types::*;

use super::*;

/// A identifier to uniquely refer to our type terms
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct TypeId(usize);

impl std::ops::Deref for TypeId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&look_up_type_id(*self).to_string())
    }
}

impl fmt::Debug for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&look_up_type_id(*self).to_string())
    }
}

impl From<usize> for TypeId {
    fn from(o: usize) -> Self {
        TypeId(o)
    }
}

impl UnresolvedTypeCheck for TypeId {
    fn check_for_unresolved_types(&self) -> Vec<CompileError> {
        use TypeInfo::*;
        let span_override = if let TypeInfo::Ref(_, span) = look_up_type_id_raw(*self) {
            Some(span)
        } else {
            None
        };
        match look_up_type_id(*self) {
            UnknownGeneric { name } => {
                panic!("here");
                vec![CompileError::UnableToInferGeneric {
                    ty: name.as_str().to_string(),
                    span: span_override.unwrap_or_else(|| name.span()),
                }]
            },
            _ => vec![],
        }
    }
}

impl TypeId {
    pub(crate) fn update_type(&mut self, type_mapping: &TypeMapping, span: &Span) {
        *self = match look_up_type_id(*self).matches_type_parameter(type_mapping) {
            Some(matching_id) => insert_type(TypeInfo::Ref(matching_id, span.clone())),
            None => {
                let ty = TypeInfo::Ref(insert_type(look_up_type_id_raw(*self)), span.clone());
                insert_type(ty)
            }
        };
    }
}

impl JsonAbiString for TypeId {
    fn json_abi_str(&self) -> String {
        look_up_type_id(*self).json_abi_str()
    }
}

impl ToJsonAbi for TypeId {
    type Output = Option<Vec<Property>>;

    fn generate_json_abi(&self) -> Self::Output {
        match look_up_type_id(*self) {
            TypeInfo::Array(type_id, _) => Some(vec![Property {
                name: "__array_element".to_string(),
                type_field: type_id.json_abi_str(),
                components: type_id.generate_json_abi(),
            }]),
            TypeInfo::Enum { variant_types, .. } => Some(
                variant_types
                    .iter()
                    .map(|x| x.generate_json_abi())
                    .collect(),
            ),
            TypeInfo::Struct { fields, .. } => {
                Some(fields.iter().map(|x| x.generate_json_abi()).collect())
            }
            TypeInfo::Tuple(fields) => Some(fields.iter().map(|x| x.generate_json_abi()).collect()),
            _ => None,
        }
    }
}
