use derivative::Derivative;
use sway_types::{Ident, Span, Spanned};

use crate::{language::ty::TyExpression, type_system::*};

#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct TyStorageDeclaration {
    pub fields: Vec<TyStorageField>,
    #[derivative(PartialEq = "ignore")]
    #[derivative(Eq(bound = ""))]
    pub span: Span,
}

impl Spanned for TyStorageDeclaration {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

#[derive(Clone, Debug, Eq)]
pub struct TyStorageField {
    pub name: Ident,
    pub type_id: TypeId,
    pub type_span: Span,
    pub initializer: TyExpression,
    pub(crate) span: Span,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TyStorageField {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && look_up_type_id(self.type_id) == look_up_type_id(other.type_id)
            && self.initializer == other.initializer
    }
}
