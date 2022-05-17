use crate::{
    parse_tree::{declaration::TypeParameter, Visibility},
    type_engine::TypeInfo,
};

use sway_types::{ident::Ident, span::Span};

#[derive(Debug, Clone)]
pub struct StructDeclaration {
    pub name: Ident,
    pub fields: Vec<StructField>,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub visibility: Visibility,
    pub(crate) span: Span,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: Ident,
    pub(crate) r#type: TypeInfo,
    pub(crate) span: Span,
    pub(crate) type_span: Span,
}
