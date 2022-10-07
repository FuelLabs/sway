use crate::{
    ast_transformation::convert_parse_tree::AttributesMap, language::Visibility, type_system::*,
};
use sway_types::{ident::Ident, span::Span};

#[derive(Debug, Clone)]
pub struct EnumDeclaration {
    pub name: Ident,
    pub attributes: AttributesMap,
    pub type_parameters: Vec<TypeParameter>,
    pub variants: Vec<EnumVariant>,
    pub(crate) span: Span,
    pub visibility: Visibility,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: Ident,
    pub attributes: AttributesMap,
    pub type_info: TypeInfo,
    pub type_span: Span,
    pub(crate) tag: usize,
    pub(crate) span: Span,
}
