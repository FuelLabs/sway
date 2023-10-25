use crate::language::parsed;
use sway_types::{ident::Ident, Span, Spanned};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TyUseStatement {
    pub call_path: Vec<Ident>,
    pub span: Span,
    pub import_type: parsed::ImportType,
    // If `is_absolute` is true, then this use statement is an absolute path from
    // the project root namespace. If not, then it is relative to the current namespace.
    pub is_absolute: bool,
    pub alias: Option<Ident>,
}

impl Spanned for TyUseStatement {
    fn span(&self) -> Span {
        self.span.clone()
    }
}
