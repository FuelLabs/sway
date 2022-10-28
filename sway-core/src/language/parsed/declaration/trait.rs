use super::{FunctionDeclaration, FunctionParameter};

use crate::{language::*, transform, type_system::*};
use sway_types::{ident::Ident, span::Span, Spanned};

#[derive(Debug, Clone)]
pub struct TraitDeclaration {
    pub name: Ident,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub attributes: transform::AttributesMap,
    pub interface_surface: Vec<TraitFn>,
    pub methods: Vec<FunctionDeclaration>,
    pub(crate) supertraits: Vec<Supertrait>,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Supertrait {
    pub name: CallPath,
}

impl Spanned for Supertrait {
    fn span(&self) -> Span {
        self.name.span()
    }
}

#[derive(Debug, Clone)]
pub struct TraitFn {
    pub name: Ident,
    pub attributes: transform::AttributesMap,
    pub purity: Purity,
    pub inline: Inline,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: TypeInfo,
    pub return_type_span: Span,
}
