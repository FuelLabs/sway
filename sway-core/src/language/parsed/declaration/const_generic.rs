use crate::TypeId;
use sway_types::{Ident, Span};

#[derive(Debug, Clone)]
pub struct ConstGenericDeclaration {
    pub name: Ident,
    pub ty: TypeId,
    pub span: Span,
}
