use crate::TypeArgument;
use sway_types::{Ident, Span};

#[derive(Debug, Clone)]
pub struct ConstGenericDeclaration {
    pub name: Ident,
    pub ty: TypeArgument,
    pub span: Span,
}
