use crate::{
    language::{parsed::*, *},
    transform,
    type_system::*,
};
use sway_types::{ident::Ident, span::Span};

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub purity: Purity,
    pub inline: Inline,
    pub attributes: transform::AttributesMap,
    pub name: Ident,
    pub visibility: Visibility,
    pub body: CodeBlock,
    pub parameters: Vec<FunctionParameter>,
    pub span: Span,
    pub return_type: TypeInfo,
    pub type_parameters: Vec<TypeParameter>,
    pub return_type_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionParameter {
    pub name: Ident,
    pub is_reference: bool,
    pub is_mutable: bool,
    pub mutability_span: Span,
    pub type_info: TypeInfo,
    pub type_span: Span,
}
