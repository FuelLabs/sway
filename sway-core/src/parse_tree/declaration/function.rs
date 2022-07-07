use crate::{parse_tree::*, type_engine::*};

use sway_types::{ident::Ident, span::Span};

mod purity;
pub use purity::{promote_purity, Purity};

mod calling_context;
pub use calling_context::{promote_calling_context, CallingContext};

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub context: CallingContext,
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
    pub(crate) type_span: Span,
}
