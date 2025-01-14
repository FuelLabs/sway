use crate::{language::ty::TyDeclParsedType, TypeId};
use sway_types::{Ident, Named, Span, Spanned};

#[derive(Debug, Clone)]
pub struct ConstGenericDeclaration {
    pub name: Ident,
    pub ty: TypeId,
    pub span: Span,
}
