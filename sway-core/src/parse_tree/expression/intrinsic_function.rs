use crate::{type_engine::TypeInfo, Expression};
use sway_types::Span;

#[derive(Debug, Clone)]
pub enum IntrinsicFunctionKind {
    SizeOfVal {
        exp: Box<Expression>,
    },
    GetPropertyOfType {
        kind: GetPropertyOfTypeKind,
        type_name: TypeInfo,
        type_span: Span,
    },
    GetStorageKey,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GetPropertyOfTypeKind {
    SizeOfType,
    IsRefType,
}
