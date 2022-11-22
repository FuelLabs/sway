use crate::{
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

impl EqWithTypeEngine for FunctionParameter {}
impl PartialEqWithTypeEngine for FunctionParameter {
    fn eq(&self, rhs: &Self, type_engine: &TypeEngine) -> bool {
        self.name == rhs.name
            && self.is_reference == rhs.is_reference
            && self.is_mutable == rhs.is_mutable
            && self.mutability_span == rhs.mutability_span
            && self.type_info.eq(&rhs.type_info, type_engine)
            && self.type_span == rhs.type_span
    }
}
