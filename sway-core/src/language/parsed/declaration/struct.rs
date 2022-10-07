use crate::{
    ast_transformation::convert_parse_tree::AttributesMap,
    language::Visibility,
    type_system::{TypeInfo, TypeParameter},
};
use sway_types::{ident::Ident, span::Span};

#[derive(Debug, Clone)]
pub struct StructDeclaration {
    pub name: Ident,
    pub attributes: AttributesMap,
    pub fields: Vec<StructField>,
    pub type_parameters: Vec<TypeParameter>,
    pub visibility: Visibility,
    pub(crate) span: Span,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: Ident,
    pub attributes: AttributesMap,
    pub type_info: TypeInfo,
    pub(crate) span: Span,
    pub type_span: Span,
}
