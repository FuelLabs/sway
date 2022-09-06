use super::{FunctionDeclaration, FunctionParameter};

use crate::{
    function::Purity,
    parse_tree::{CallPath, Visibility},
    type_system::TypeInfo,
};

use sway_types::{ident::Ident, span::Span, Spanned};

#[derive(Debug, Clone)]
pub struct TraitDeclaration {
    pub name: Ident,
    pub interface_surface: Vec<TraitFn>,
    pub methods: Vec<FunctionDeclaration>,
    pub(crate) supertraits: Vec<Supertrait>,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct Supertrait {
    pub(crate) name: CallPath,
}

impl Spanned for Supertrait {
    fn span(&self) -> Span {
        self.name.span()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TraitFn {
    pub name: Ident,
    pub purity: Purity,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: TypeInfo,
    pub return_type_span: Span,
}
