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

use sway_types::{constants::ALLOW_DEAD_CODE_NAME, Ident, Span};

use std::{collections::HashMap, hash::Hash, sync::Arc};

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
    DocComment,
    Storage,
    Inline,
    Test,
    Payable,
    Allow,
}

impl AttributeKind {
    // Returns tuple with the mininum and maximum number of expected args
    // None can be returned in the second position of the tuple if there is no maximum
    pub fn expected_args_len_min_max(self) -> (usize, Option<usize>) {
        match self {
            AttributeKind::Doc => (0, None),
            AttributeKind::DocComment => (0, None),
            AttributeKind::Storage => (0, None),
            AttributeKind::Inline => (0, None),
            AttributeKind::Test => (0, None),
            AttributeKind::Payable => (0, None),
            AttributeKind::Allow => (1, Some(1)),
        }
    }

    // Returns the expected values for an attribute argument
    pub fn expected_args_values(self, _arg_index: usize) -> Option<Vec<String>> {
        match self {
            AttributeKind::Doc => None,
            AttributeKind::DocComment => None,
            AttributeKind::Storage => None,
            AttributeKind::Inline => None,
            AttributeKind::Test => None,
            AttributeKind::Payable => None,
            AttributeKind::Allow => Some(vec![ALLOW_DEAD_CODE_NAME.to_string()]),
        }
    }
}

/// Stores the attributes associated with the type.
#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct AttributesMap(Arc<HashMap<AttributeKind, Vec<Attribute>>>);

impl AttributesMap {
    /// Create a new attributes map.
    pub fn new(attrs_map: Arc<HashMap<AttributeKind, Vec<Attribute>>>) -> AttributesMap {
        AttributesMap(attrs_map)
    }

    /// Returns the first attribute by span, or None if there are no attributes.
    pub fn first(&self) -> Option<(&AttributeKind, &Attribute)> {
        let mut first: Option<(&AttributeKind, &Attribute)> = None;
        for (kind, attrs) in self.iter() {
            for attr in attrs {
                if let Some((_, first_attr)) = first {
                    if attr.span.start() < first_attr.span.start() {
                        first = Some((kind, attr));
                    }
                } else {
                    first = Some((kind, attr));
                }
            }
        }
        first
    }

    pub fn inner(&self) -> &HashMap<AttributeKind, Vec<Attribute>> {
        &self.0
    }
}

impl std::ops::Deref for AttributesMap {
    type Target = Arc<HashMap<AttributeKind, Vec<Attribute>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
