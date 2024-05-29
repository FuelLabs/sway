use crate::{
    engine_threading::{EqWithEngines, PartialEqWithEngines, PartialEqWithEnginesContext},
    language::parsed::Expression,
    transform,
    type_system::*,
};
use sway_types::{ident::Ident, span::Span, Spanned};

#[derive(Debug, Clone)]
/// A declaration of contract storage. Only valid within contract contexts.
/// All values in this struct are mutable and persistent among executions of the same contract deployment.
pub struct StorageDeclaration {
    pub attributes: transform::AttributesMap,
    pub entries: Vec<StorageEntry>,
    pub span: Span,
    pub storage_keyword: Ident,
}

impl EqWithEngines for StorageDeclaration {}
impl PartialEqWithEngines for StorageDeclaration {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.attributes == other.attributes
            && self.fields.eq(&other.fields, ctx)
            && self.span == other.span
            && self.storage_keyword == other.storage_keyword
    }
}

impl Spanned for StorageDeclaration {
    fn span(&self) -> sway_types::Span {
        self.span.clone()
    }
}

#[derive(Debug, Clone)]
pub struct StorageNamespace {
    pub name: Ident,
    pub entries: Vec<Box<StorageEntry>>,
}

#[derive(Debug, Clone)]
pub enum StorageEntry {
    Namespace(StorageNamespace),
    Field(StorageField),
}

impl StorageEntry {
    pub fn name(&self) -> Ident {
        match self {
            StorageEntry::Namespace(namespace) => namespace.name.clone(),
            StorageEntry::Field(field) => field.name.clone(),
        }
    }
}

/// An individual field in a storage declaration.
/// A type annotation _and_ initializer value must be provided. The initializer value must be a
/// constant expression. For now, that basically means just a literal, but as constant folding
/// improves, we can update that.
#[derive(Debug, Clone)]
pub struct StorageField {
    pub name: Ident,
    pub key_expression: Option<Expression>,
    pub attributes: transform::AttributesMap,
    pub type_argument: TypeArgument,
    pub span: Span,
    pub initializer: Expression,
}

impl EqWithEngines for StorageField {}
impl PartialEqWithEngines for StorageField {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name == other.name
            && self.attributes == other.attributes
            && self.type_argument.eq(&other.type_argument, ctx)
            && self.span == other.span
            && self.initializer.eq(&other.initializer, ctx)
    }
}
