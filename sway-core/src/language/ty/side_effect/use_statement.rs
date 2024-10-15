use crate::language::parsed;
use sway_types::{ident::Ident, Span, Spanned};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TyUseStatement {
    pub call_path: Vec<Ident>,
    pub span: Span,
    pub import_type: parsed::ImportType,
    // If `is_relative_to_package_root` is true, then this use statement is a path relative to the
    // project root. For example, if the path is `::X::Y` and occurs in package `P`, then the path
    // refers to the full path `P::X::Y`.
    // If `is_relative_to_package_root` is false, then there are two options:
    // - The path refers to a path relative to the current namespace. For example, if the path is
    //   `X::Y` and it occurs in a module whose path is `P::M`, then the path refers to the full
    //   path `P::M::X::Y`.
    // - The path refers to a path in an external package. For example, the path `X::Y` refers to an
    //   entity `Y` in the external package `X`.
    pub is_relative_to_package_root: bool,
    pub alias: Option<Ident>,
}

impl Spanned for TyUseStatement {
    fn span(&self) -> Span {
        self.span.clone()
    }
}
