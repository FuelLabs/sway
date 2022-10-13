use super::FunctionDeclaration;
use crate::{
    language::CallPath,
    type_system::{TypeInfo, TypeParameter},
};

use sway_types::span::Span;

#[derive(Debug, Clone)]
pub struct ImplTrait {
    pub impl_type_parameters: Vec<TypeParameter>,
    pub trait_name: CallPath,
    pub trait_type_parameters: Vec<TypeParameter>,
    pub type_implementing_for: TypeInfo,
    pub type_implementing_for_span: Span,
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
    pub type_parameters: Vec<TypeParameter>,
    pub functions: Vec<FunctionDeclaration>,
    // the span of the whole impl trait and block
    pub(crate) block_span: Span,
}
