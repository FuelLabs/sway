use crate::{parse_tree::*, type_system::*};

use sway_types::{ident::Ident, span::Span};

mod purity;
pub use purity::{promote_purity, Purity};

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub purity: Purity,
    pub name: Ident,
    pub visibility: Visibility,
    pub body: CodeBlock,
    pub parameters: Vec<FunctionParameter>,
    pub span: Span,
    pub return_type: TypeInfo,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub(crate) return_type_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionParameter {
    pub name: Ident,
    pub is_mutable: bool,
    pub(crate) type_id: TypeId,
    pub type_span: Span,
}
