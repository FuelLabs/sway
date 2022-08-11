use crate::{parse_tree::Expression, type_system::*};

use sway_types::{ident::Ident, span::Span};

#[derive(Debug, Clone)]
/// A declaration of contract storage. Only valid within contract contexts.
/// All values in this struct are mutable and persistent among executions of the same contract deployment.
pub struct StorageDeclaration {
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
    pub type_info: TypeInfo,
    pub initializer: Expression,
}
