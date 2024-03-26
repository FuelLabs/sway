use crate::{language::Visibility, transform, type_system::*};
use sway_types::{ident::Ident, span::Span};

#[derive(Debug, Clone, deepsize::DeepSizeOf)]
pub struct EnumDeclaration {
    pub name: Ident,
    pub attributes: transform::AttributesMap,
    pub type_parameters: Vec<TypeParameter>,
    pub variants: Vec<EnumVariant>,
    pub(crate) span: Span,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, deepsize::DeepSizeOf)]
pub struct EnumVariant {
    pub name: Ident,
    pub attributes: transform::AttributesMap,
    pub type_argument: TypeArgument,
    pub(crate) tag: usize,
    pub(crate) span: Span,
}
