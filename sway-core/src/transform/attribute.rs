//! Each item may have a list of attributes, each with a name (the key to the hashmap) and a list of
//! zero or more args.  Attributes may be specified more than once in which case we use the union of
//! their args.
//!
//! E.g.,
//!
//!   #[foo(bar)]
//!   #[foo(baz, xyzzy)]
//!
//! is essentially equivalent to
//!
//!   #[foo(bar, baz, xyzzy)]
//!
//! but no uniquing is done so
//!
//!   #[foo(bar)]
//!   #[foo(bar)]
//!
//! is
//!
//!   #[foo(bar, bar)]

use std::{collections::HashMap, sync::Arc};
use sway_types::{Ident, Span};

/// An attribute has a name (i.e "doc", "storage"),
/// a vector of possible arguments and
/// a span from its declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attribute {
    pub name: Ident,
    pub args: Vec<Ident>,
    pub span: Span,
}

/// Valid kinds of attributes supported by the compiler
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum AttributeKind {
    Doc,
    Storage,
    Inline,
    Test,
    Payable,
}

/// Stores the attributes associated with the type.
pub type AttributesMap = Arc<HashMap<AttributeKind, Vec<Attribute>>>;
