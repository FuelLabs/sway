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
//! and duplicates like
//!
//!   #[foo(bar)]
//!   #[foo(bar)]
//!
//! are equivalent to
//!
//!   #[foo(bar, bar)]

use indexmap::IndexMap;
use sway_ast::Literal;
use sway_types::{
    constants::{
        ALLOW_DEAD_CODE_NAME, ALLOW_DEPRECATED_NAME, CFG_EXPERIMENTAL_NEW_ENCODING,
        CFG_PROGRAM_TYPE_ARG_NAME, CFG_TARGET_ARG_NAME,
    },
    Ident, Span, Spanned,
};

use std::{hash::Hash, sync::Arc};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributeArg {
    pub name: Ident,
    pub value: Option<Literal>,
    pub span: Span,
}

impl Spanned for AttributeArg {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

/// An attribute has a name (i.e "doc", "storage"),
/// a vector of possible arguments and
/// a span from its declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attribute {
    pub name: Ident,
    pub args: Vec<AttributeArg>,
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
    Cfg,
    Deprecated,
    Namespace,
    Fallback,
}

impl AttributeKind {
    // Returns tuple with the minimum and maximum number of expected args
    // None can be returned in the second position of the tuple if there is no maximum
    pub fn expected_args_len_min_max(self) -> (usize, Option<usize>) {
        use AttributeKind::*;
        match self {
            Doc | DocComment | Storage | Inline | Test | Payable | Deprecated | Fallback => {
                (0, None)
            }
            Allow | Cfg | Namespace => (1, Some(1)),
        }
    }

    // Returns the expected values for an attribute argument
    pub fn expected_args_values(self, _arg_index: usize) -> Option<Vec<String>> {
        use AttributeKind::*;
        match self {
            Deprecated | Namespace | Doc | DocComment | Storage | Inline | Test | Payable
            | Fallback => None,
            Allow => Some(vec![
                ALLOW_DEAD_CODE_NAME.to_string(),
                ALLOW_DEPRECATED_NAME.to_string(),
            ]),
            Cfg => Some(vec![
                CFG_TARGET_ARG_NAME.to_string(),
                CFG_PROGRAM_TYPE_ARG_NAME.to_string(),
                CFG_EXPERIMENTAL_NEW_ENCODING.to_string(),
            ]),
        }
    }
}

/// Stores the attributes associated with the type.
#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct AttributesMap(Arc<IndexMap<AttributeKind, Vec<Attribute>>>);

impl AttributesMap {
    /// Create a new attributes map.
    pub fn new(attrs_map: Arc<IndexMap<AttributeKind, Vec<Attribute>>>) -> AttributesMap {
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

    pub fn inner(&self) -> &IndexMap<AttributeKind, Vec<Attribute>> {
        &self.0
    }
}

impl std::ops::Deref for AttributesMap {
    type Target = Arc<IndexMap<AttributeKind, Vec<Attribute>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct AllowDeprecatedEnterToken {
    diff: i32,
}

#[derive(Default)]
pub struct AllowDeprecatedState {
    allowed: u32,
}
impl AllowDeprecatedState {
    pub(crate) fn enter(&mut self, attributes: AttributesMap) -> AllowDeprecatedEnterToken {
        if let Some(all_allows) = attributes.get(&AttributeKind::Allow) {
            for allow in all_allows {
                for arg in allow.args.iter() {
                    if arg.name.as_str() == ALLOW_DEPRECATED_NAME {
                        self.allowed += 1;
                        return AllowDeprecatedEnterToken { diff: -1 };
                    }
                }
            }
        }

        AllowDeprecatedEnterToken { diff: 0 }
    }

    pub(crate) fn exit(&mut self, token: AllowDeprecatedEnterToken) {
        self.allowed = self.allowed.saturating_add_signed(token.diff);
    }

    pub(crate) fn is_allowed(&self) -> bool {
        self.allowed > 0
    }
}
