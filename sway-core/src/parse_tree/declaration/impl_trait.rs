use super::FunctionDeclaration;
use crate::{
    parse_tree::CallPath,
    type_system::{TypeInfo, TypeParameter},
};

use sway_types::span::Span;

#[derive(Debug, Clone)]
pub struct ImplTrait {
    pub trait_name: CallPath,
    pub type_implementing_for: TypeInfo,
    pub type_implementing_for_span: Span,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub functions: Vec<FunctionDeclaration>,
    // the span of the whole impl trait and block
    pub(crate) block_span: Span,
}

/// An impl of methods without a trait
/// like `impl MyType { fn foo { .. } }`
#[derive(Debug, Clone)]
pub struct ImplSelf {
    pub type_implementing_for: TypeInfo,
    pub(crate) type_implementing_for_span: Span,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub functions: Vec<FunctionDeclaration>,
    // the span of the whole impl trait and block
    pub(crate) block_span: Span,
}
