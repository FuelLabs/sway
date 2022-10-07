use sway_types::ident::Ident;

#[derive(Debug, Clone)]
pub enum ImportType {
    Star,
    SelfImport,
    Item(Ident),
}

/// A [UseStatement] is a statement that imports something from a module into the local namespace.
#[derive(Debug, Clone)]
pub struct UseStatement {
    pub(crate) call_path: Vec<Ident>,
    pub(crate) import_type: ImportType,
    // If `is_absolute` is true, then this use statement is an absolute path from
    // the project root namespace. If not, then it is relative to the current namespace.
    pub(crate) is_absolute: bool,
    pub(crate) alias: Option<Ident>,
}
