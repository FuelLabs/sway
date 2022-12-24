use crate::{
    engine_threading::*,
    language::{parsed::*, *},
    transform,
    type_system::*,
};
use sway_types::{ident::Ident, span::Span};

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub purity: Purity,
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

#[derive(Debug, Clone)]
pub struct FunctionParameter {
    pub name: Ident,
    pub is_reference: bool,
    pub is_mutable: bool,
    pub mutability_span: Span,
    pub type_info: TypeInfo,
    pub type_span: Span,
}

impl EqWithEngines for FunctionParameter {}
impl PartialEqWithEngines for FunctionParameter {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.name == other.name
            && self.is_reference == other.is_reference
            && self.is_mutable == other.is_mutable
            && self.mutability_span == other.mutability_span
            && self.type_info.eq(&other.type_info, engines)
            && self.type_span == other.type_span
    }
}
