use super::{FunctionDeclaration, FunctionParameter};

use crate::{
    build_config::BuildConfig,
    error::*,
    parse_tree::{ident, CallPath, Visibility},
    style::{is_snake_case, is_upper_camel_case},
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
    pub parameters: Vec<FunctionParameter>,
    pub return_type: TypeInfo,
    pub(crate) return_type_span: Span,
}
