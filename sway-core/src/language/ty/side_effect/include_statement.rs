use crate::language::Visibility;
use serde::{Serialize, Deserialize};
use sway_types::{ident::Ident, Span, Spanned};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TyIncludeStatement {
    pub span: Span,
    pub visibility: Visibility,
    pub mod_name: Ident,
}

impl Spanned for TyIncludeStatement {
    fn span(&self) -> Span {
        self.span.clone()
    }
}
