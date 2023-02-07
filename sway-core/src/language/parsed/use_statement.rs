use crate::parsed::Span;
use sway_types::ident::Ident;

#[derive(Debug, Clone)]
pub enum ImportType {
    Star,
    SelfImport(Span),
    Item(Ident),
}

/// A [UseStatement] is a statement that imports something from a module into the local namespace.
#[derive(Debug, Clone)]
pub struct UseStatement {
    pub call_path: Vec<Ident>,
    pub import_type: ImportType,
    // If `is_absolute` is true, then this use statement is an absolute path from
    // the project root namespace. If not, then it is relative to the current namespace.
    pub is_absolute: bool,
    pub alias: Option<Ident>,
}
