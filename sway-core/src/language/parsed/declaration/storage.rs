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
    pub attributes: transform::Attributes,
    pub entries: Vec<StorageEntry>,
    pub span: Span,
    pub storage_keyword: Ident,
}

impl EqWithEngines for StorageDeclaration {}
impl PartialEqWithEngines for StorageDeclaration {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.attributes == other.attributes
            && self.entries.eq(&other.entries, ctx)
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

impl EqWithEngines for StorageNamespace {}
impl PartialEqWithEngines for StorageNamespace {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name.eq(&other.name) && self.entries.eq(&other.entries, ctx)
    }
}

#[allow(clippy::large_enum_variant)]
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

impl EqWithEngines for StorageEntry {}
impl PartialEqWithEngines for StorageEntry {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        match (self, other) {
            (StorageEntry::Namespace(n1), StorageEntry::Namespace(n2)) => n1.eq(n2, ctx),
            (StorageEntry::Field(f1), StorageEntry::Field(f2)) => f1.eq(f2, ctx),
            _ => false,
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
    pub attributes: transform::Attributes,
    pub type_argument: GenericTypeArgument,
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
