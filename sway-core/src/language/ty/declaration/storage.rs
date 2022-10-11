use derivative::Derivative;
use sway_types::{Span, Spanned};

use crate::{semantic_analysis::TyStorageField, AttributesMap};

#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct TyStorageDeclaration {
    pub fields: Vec<TyStorageField>,
    #[derivative(PartialEq = "ignore")]
    #[derivative(Eq(bound = ""))]
    pub span: Span,
    pub attributes: AttributesMap,
}

impl Spanned for TyStorageDeclaration {
    fn span(&self) -> Span {
        self.span.clone()
    }
}
