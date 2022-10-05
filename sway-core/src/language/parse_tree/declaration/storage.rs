use crate::{language::parse_tree::Expression, type_system::*, AttributesMap};
use sway_types::{ident::Ident, span::Span};

#[derive(Debug, Clone)]
/// A declaration of contract storage. Only valid within contract contexts.
/// All values in this struct are mutable and persistent among executions of the same contract deployment.
pub struct StorageDeclaration {
    pub attributes: AttributesMap,
    pub fields: Vec<StorageField>,
    pub span: Span,
}

/// An individual field in a storage declaration.
/// A type annotation _and_ initializer value must be provided. The initializer value must be a
/// constant expression. For now, that basically means just a literal, but as constant folding
/// improves, we can update that.
#[derive(Debug, Clone)]
pub struct StorageField {
    pub name: Ident,
    pub attributes: AttributesMap,
    pub type_info: TypeInfo,
    pub type_info_span: Span,
    pub initializer: Expression,
}
