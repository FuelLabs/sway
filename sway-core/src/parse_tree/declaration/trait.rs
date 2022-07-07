use super::{FunctionDeclaration, FunctionParameter};

use crate::{
    function::{CallingContext, Purity},
    parse_tree::{CallPath, Visibility},
    type_engine::TypeInfo,
};

use sway_types::{ident::Ident, span::Span};

#[derive(Debug, Clone)]
pub struct TraitDeclaration {
    pub name: Ident,
    pub(crate) interface_surface: Vec<TraitFn>,
    pub methods: Vec<FunctionDeclaration>,
    pub(crate) supertraits: Vec<Supertrait>,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct Supertrait {
    pub(crate) name: CallPath,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TraitFn {
    pub name: Ident,
    pub purity: Purity,
    pub context: CallingContext,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: TypeInfo,
    pub(crate) return_type_span: Span,
}
