use crate::{language::Visibility, parsed::Span};
use serde::{Serialize, Deserialize};
use sway_types::ident::Ident;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImportType {
    Star,
    SelfImport(Span),
    Item(Ident),
}

/// A [UseStatement] is a statement that imports something from a module into the local namespace.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UseStatement {
    pub call_path: Vec<Ident>,
    pub span: Span,
    pub import_type: ImportType,
    // If `is_absolute` is true, then this use statement is an absolute path from
    // the project root namespace. If not, then it is relative to the current namespace.
    pub is_absolute: bool,
    // If `reexport` is Visibility::Public, then this use statement reexports its imported binding.
    // If not, then the import binding is private to the importing module.
    pub reexport: Visibility,
    pub alias: Option<Ident>,
}
