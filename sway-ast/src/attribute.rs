use crate::priv_prelude::*;

#[derive(Clone, Debug, Serialize)]
pub struct Annotated<T> {
    pub attributes: Vec<AttributeDecl>,
    pub value: T,
}

// Storage access and purity.
pub const STORAGE_ATTRIBUTE_NAME: &str = "storage";
pub const STORAGE_READ_ARG_NAME: &str = "read";
pub const STORAGE_WRITE_ARG_NAME: &str = "write";

// Function inlining.
pub const INLINE_ATTRIBUTE_NAME: &str = "inline";
pub const INLINE_NEVER_ARG_NAME: &str = "never";
pub const INLINE_ALWAYS_ARG_NAME: &str = "always";

// Payable functions.
pub const PAYABLE_ATTRIBUTE_NAME: &str = "payable";

// Fallback functions.
pub const FALLBACK_ATTRIBUTE_NAME: &str = "fallback";

// Documentation comments.
// Note that because "doc-comment" is not a valid identifier,
// doc-comment attributes cannot be declared in code.
// They are exclusively created by the compiler to denote
// doc comments, `///` and `//!`.
pub const DOC_COMMENT_ATTRIBUTE_NAME: &str = "doc-comment";

// In-language unit testing.
pub const TEST_ATTRIBUTE_NAME: &str = "test";
pub const TEST_SHOULD_REVERT_ARG_NAME: &str = "should_revert";

// In-language fuzz testing.
pub const FUZZ_ATTRIBUTE_NAME: &str = "fuzz";
pub const FUZZ_PARAM_ATTRIBUTE_NAME: &str = "fuzz_param";
pub const FUZZ_PARAM_NAME_ARG_NAME: &str = "name";
pub const FUZZ_PARAM_ITERATION_ARG_NAME: &str = "iteration";
pub const FUZZ_PARAM_MIN_VAL_ARG_NAME: &str = "min_val";
pub const FUZZ_PARAM_MAX_VAL_ARG_NAME: &str = "max_val";

// Allow warnings.
pub const ALLOW_ATTRIBUTE_NAME: &str = "allow";
pub const ALLOW_DEAD_CODE_ARG_NAME: &str = "dead_code";
pub const ALLOW_DEPRECATED_ARG_NAME: &str = "deprecated";

// Conditional compilation.
pub const CFG_ATTRIBUTE_NAME: &str = "cfg";
pub const CFG_TARGET_ARG_NAME: &str = "target";
pub const CFG_PROGRAM_TYPE_ARG_NAME: &str = "program_type";

// Deprecation.
pub const DEPRECATED_ATTRIBUTE_NAME: &str = "deprecated";
pub const DEPRECATED_NOTE_ARG_NAME: &str = "note";

// Error types.
pub const ERROR_TYPE_ATTRIBUTE_NAME: &str = "error_type";
pub const ERROR_ATTRIBUTE_NAME: &str = "error";
pub const ERROR_M_ARG_NAME: &str = "m";

// Backtracing.
pub const TRACE_ATTRIBUTE_NAME: &str = "trace";
pub const TRACE_NEVER_ARG_NAME: &str = "never";
pub const TRACE_ALWAYS_ARG_NAME: &str = "always";

// Abi names.
pub const ABI_NAME_ATTRIBUTE_NAME: &str = "abi_name";
pub const ABI_NAME_NAME_ARG_NAME: &str = "name";

pub const KNOWN_ATTRIBUTE_NAMES: &[&str] = &[
    STORAGE_ATTRIBUTE_NAME,
    DOC_COMMENT_ATTRIBUTE_NAME,
    TEST_ATTRIBUTE_NAME,
    INLINE_ATTRIBUTE_NAME,
    PAYABLE_ATTRIBUTE_NAME,
    ALLOW_ATTRIBUTE_NAME,
    CFG_ATTRIBUTE_NAME,
    DEPRECATED_ATTRIBUTE_NAME,
    FALLBACK_ATTRIBUTE_NAME,
    ABI_NAME_ATTRIBUTE_NAME,
    FUZZ_ATTRIBUTE_NAME,
    FUZZ_PARAM_ATTRIBUTE_NAME,
];

/// An attribute declaration. Attribute declaration
/// can potentially have an arbitrary number of [Attribute]s.
/// [Attribute]s can potentially have any number of [AttributeArg]s.
/// Each [AttributeArg] can have a value assigned.
///
/// E.g.:
///
/// ```ignore
/// #[attribute]
/// #[attribute_1, attribute_2]
/// #[attribute()]
/// #[attribute(arg)]
/// #[attribute(arg_1, arg_2)]
/// #[attribute(arg_1 = "value", arg_2 = true)]
/// #[attribute_1, attribute_2(arg_1), attribute_3(arg_1, arg_2 = true)]
/// ```
///
/// [AttributeDecl]s can be _inner_ or _outer_, as explained in [AttributeHashKind].
// TODO: Currently, inner attributes are supported only on module doc comments,
//       those starting with `//!`.
//       See: https://github.com/FuelLabs/sway/issues/6924
#[derive(Clone, Debug, Serialize)]
pub struct AttributeDecl {
    pub hash_kind: AttributeHashKind,
    pub attribute: SquareBrackets<Punctuated<Attribute, CommaToken>>,
}

impl AttributeDecl {
    /// Creates the `doc-comment` [AttributeDecl] for a single line of an outer comment. E.g.:
    /// ```ignore
    /// /// This is an outer comment.
    /// ```
    /// The `span` is the overall span: `/// This is an outer comment.`.
    /// The `content_span` is the span of the content, without the leading `///`: ` This is an outer comment.`.
    pub fn new_outer_doc_comment(span: Span, content_span: Span) -> Self {
        Self::new_doc_comment(
            span.clone(),
            content_span,
            AttributeHashKind::Outer(HashToken::new(span)),
        )
    }

    /// Creates the `doc-comment` [AttributeDecl] for a single line of an inner comment. E.g.:
    /// ```ignore
    /// //! This is an inner comment.
    /// ```
    /// The `span` is the overall span: `//! This is an inner comment.`.
    /// The `content_span` is the span of the content, without the leading `//!`: ` This is an inner comment.`.
    pub fn new_inner_doc_comment(span: Span, content_span: Span) -> Self {
        Self::new_doc_comment(
            span.clone(),
            content_span,
            AttributeHashKind::Inner(HashBangToken::new(span)),
        )
    }

    fn new_doc_comment(span: Span, content_span: Span, hash_kind: AttributeHashKind) -> Self {
        // TODO: Store the comment line in an argument value as
        //       discussed in https://github.com/FuelLabs/sway/issues/6938.
        let name = Ident::new_no_trim(content_span.clone());
        AttributeDecl {
            hash_kind,
            attribute: SquareBrackets::new(
                Punctuated::single(Attribute {
                    name: Ident::new_with_override(
                        DOC_COMMENT_ATTRIBUTE_NAME.to_string(),
                        span.clone(),
                    ),
                    args: Some(Parens::new(
                        Punctuated::single(AttributeArg { name, value: None }),
                        content_span,
                    )),
                }),
                span,
            ),
        }
    }

    /// `self` is a doc comment, either an inner (`//!`) or outer (`///`).
    pub fn is_doc_comment(&self) -> bool {
        self.attribute.inner.value_separator_pairs.is_empty()
            && self
                .attribute
                .inner
                .final_value_opt
                .as_ref()
                .is_some_and(|attr| attr.is_doc_comment())
    }

    pub fn is_inner(&self) -> bool {
        matches!(self.hash_kind, AttributeHashKind::Inner(_))
    }
}

impl Spanned for AttributeDecl {
    fn span(&self) -> Span {
        let hash_span = match &self.hash_kind {
            AttributeHashKind::Inner(hash_bang_token) => hash_bang_token.span(),
            AttributeHashKind::Outer(hash_token) => hash_token.span(),
        };
        Span::join(hash_span, &self.attribute.span())
    }
}

/// Denotes if an [AttributeDecl] is an _inner_ or _outer_ attribute declaration.
///
/// E.g.:
/// ```ignore
/// /// This is an outer doc comment.
/// /// It annotates the struct `Foo`.
/// struct Foo {}
///
/// // This is an outer attribute.
/// // In annotates the function `fun`.
/// #[inline(always)]
/// fn fun() {}
///
/// //! This is an inner doc comment.
/// //! It annotates the library module.
/// library;
///
/// // This is an inner attribute.
/// // In annotates whichever item it is declared in.
/// #![allow(dead_code)]
/// ```
#[derive(Clone, Debug, Serialize)]
pub enum AttributeHashKind {
    /// Inner specifies that the attribute annotates
    /// the item that the attribute is declared within.
    Inner(HashBangToken),
    /// Outer specifies that the attribute annotates
    /// the item that immediately follows the attribute.
    Outer(HashToken),
}

impl Spanned for AttributeHashKind {
    fn span(&self) -> Span {
        match self {
            AttributeHashKind::Inner(token) => token.span(),
            AttributeHashKind::Outer(token) => token.span(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct AttributeArg {
    pub name: Ident,
    pub value: Option<Literal>,
}

impl Spanned for AttributeArg {
    fn span(&self) -> Span {
        if let Some(value) = &self.value {
            Span::join(self.name.span(), &value.span())
        } else {
            self.name.span()
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Attribute {
    pub name: Ident,
    pub args: Option<Parens<Punctuated<AttributeArg, CommaToken>>>,
}

impl Attribute {
    pub fn is_doc_comment(&self) -> bool {
        self.name.as_str() == DOC_COMMENT_ATTRIBUTE_NAME
    }
    pub fn is_cfg(&self) -> bool {
        self.name.as_str() == CFG_ATTRIBUTE_NAME
    }
}

impl Spanned for Attribute {
    fn span(&self) -> Span {
        self.args
            .as_ref()
            .map(|args| Span::join(self.name.span(), &args.span()))
            .unwrap_or_else(|| self.name.span())
    }
}
