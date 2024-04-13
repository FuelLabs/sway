use crate::{language::Visibility, transform, type_system::*};
use sway_types::{ident::Ident, span::Span, Named, Spanned};

#[derive(Debug, Clone)]
pub struct EnumDeclaration {
    pub name: Ident,
    pub attributes: transform::AttributesMap,
    pub type_parameters: Vec<TypeParameter>,
    pub variants: Vec<EnumVariant>,
    pub(crate) span: Span,
    pub visibility: Visibility,
}

impl Named for EnumDeclaration {
    fn name(&self) -> &sway_types::BaseIdent {
        &self.name
    }
}

impl Spanned for EnumDeclaration {
    fn span(&self) -> sway_types::Span {
        self.span.clone()
    }
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: Ident,
    pub attributes: transform::AttributesMap,
    pub type_argument: TypeArgument,
    pub(crate) tag: usize,
    pub(crate) span: Span,
}
