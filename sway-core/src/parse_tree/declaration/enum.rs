use crate::{parse_tree::Visibility, type_system::*};

use sway_types::{ident::Ident, span::Span};

#[derive(Debug, Clone)]
pub struct EnumDeclaration {
    pub name: Ident,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub variants: Vec<EnumVariant>,
    pub(crate) span: Span,
    pub visibility: Visibility,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: Ident,
    pub type_info: TypeInfo,
    pub type_span: Span,
    pub(crate) tag: usize,
    pub(crate) span: Span,
}
