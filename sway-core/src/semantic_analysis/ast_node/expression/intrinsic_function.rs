use std::fmt;

use sway_types::Span;

use crate::{type_engine::*, types::DeterministicallyAborts, GetPropertyOfTypeKind};

use super::TypedExpression;

#[derive(Debug, Clone)]
pub enum TypedIntrinsicFunctionKind {
    SizeOfVal {
        exp: Box<TypedExpression>,
    },
    GetPropertyOfType {
        kind: GetPropertyOfTypeKind,
        type_id: TypeId,
        type_span: Span,
    },
    GetStorageKey,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypedIntrinsicFunctionKind {
    fn eq(&self, other: &Self) -> bool {
        use TypedIntrinsicFunctionKind::*;
        match (self, other) {
            (SizeOfVal { exp: l_exp }, SizeOfVal { exp: r_exp }) => *l_exp == *r_exp,
            (
                GetPropertyOfType {
                    kind: l_kind,
                    type_id: l_type_id,
                    ..
                },
                GetPropertyOfType {
                    kind: r_kind,
                    type_id: r_type_id,
                    ..
                },
            ) => l_kind == r_kind && look_up_type_id(*l_type_id) == look_up_type_id(*r_type_id),
            (GetStorageKey, GetStorageKey) => true,
            _ => false,
        }
    }
}

impl CopyTypes for TypedIntrinsicFunctionKind {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        use TypedIntrinsicFunctionKind::*;
        match self {
            SizeOfVal { exp } => {
                exp.copy_types(type_mapping);
            }
            GetPropertyOfType {
                type_id, type_span, ..
            } => {
                type_id.update_type(type_mapping, type_span);
            }
            GetStorageKey => {}
        }
    }
}

impl fmt::Display for TypedIntrinsicFunctionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TypedIntrinsicFunctionKind::*;
        let s = match self {
            SizeOfVal { exp } => format!("size_of_val({})", *exp),
            GetPropertyOfType { kind, type_id, .. } => {
                let type_str = look_up_type_id(*type_id).to_string();
                match kind {
                    GetPropertyOfTypeKind::SizeOfType => format!("size_of({type_str})"),
                    GetPropertyOfTypeKind::IsRefType => format!("is_ref_type({type_str})"),
                }
            }
            GetStorageKey => "get_storage_key".to_string(),
        };
        write!(f, "{}", s)
    }
}

impl DeterministicallyAborts for TypedIntrinsicFunctionKind {
    fn deterministically_aborts(&self) -> bool {
        match self {
            TypedIntrinsicFunctionKind::SizeOfVal { exp } => exp.deterministically_aborts(),
            TypedIntrinsicFunctionKind::GetPropertyOfType { .. }
            | TypedIntrinsicFunctionKind::GetStorageKey => false,
        }
    }
}
