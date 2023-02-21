use crate::{language::Visibility, transform, type_system::TypeParam, TypeArgument};
use sway_types::{ident::Ident, span::Span};

#[derive(Debug, Clone)]
pub struct StructDeclaration {
    pub name: Ident,
    pub attributes: transform::AttributesMap,
    pub fields: Vec<StructField>,
    pub type_parameters: Vec<TypeParam>,
    pub visibility: Visibility,
    pub(crate) span: Span,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: Ident,
    pub attributes: transform::AttributesMap,
    pub(crate) span: Span,
    pub type_argument: TypeArgument,
}
