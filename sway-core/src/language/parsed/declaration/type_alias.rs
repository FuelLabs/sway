use crate::{language::Visibility, transform, type_system::*};

use sway_types::{ident::Ident, span::Span, Named, Spanned};

#[derive(Debug, Clone)]
pub struct TypeAliasDeclaration {
    pub name: Ident,
    pub attributes: transform::AttributesMap,
    pub ty: TypeArgument,
    pub visibility: Visibility,
    pub span: Span,
}

impl Named for TypeAliasDeclaration {
    fn name(&self) -> &sway_types::BaseIdent {
        &self.name
    }
}

impl Spanned for TypeAliasDeclaration {
    fn span(&self) -> sway_types::Span {
        self.span.clone()
    }
}
