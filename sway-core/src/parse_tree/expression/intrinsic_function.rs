use crate::{type_engine::TypeInfo, Expression};
use sway_types::Span;

#[derive(Debug, Clone)]
pub enum IntrinsicFunctionKind {
    SizeOfVal {
        exp: Box<Expression>,
    },
    SizeOfType {
        type_name: TypeInfo,
        type_span: Span,
    },
    IsRefType {
        type_name: TypeInfo,
        type_span: Span,
    },
    GetStorageKey,
}
