use crate::{language::Visibility, transform, type_system::*};

use sway_types::{ident::Ident, span::Span};

#[derive(Debug, Clone)]
pub struct TypeAliasDeclaration {
    pub name: Ident,
    pub attributes: transform::AttributesMap,
    pub ty: TypeArgument,
    pub visibility: Visibility,
    pub span: Span,
}
