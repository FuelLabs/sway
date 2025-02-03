use crate::priv_prelude::*;

#[derive(Clone, Debug, Serialize)]
pub struct Annotated<T> {
    // TODO-IG!: Rename to attributes. Search replace in unit tests.
    pub attribute_list: Vec<AttributeDecl>,
    pub value: T,
}

// TODO-IG!: Check if this is the best place to put all the constants.
pub const DOC_COMMENT_ATTRIBUTE_NAME: &str = "doc-comment";
pub const CFG_ATTRIBUTE_NAME: &str = "cfg";

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
    // TODO-IG!: Rename to AttributeDirection. Check Rust terminology.
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
        Self::new_doc_comment(span.clone(), content_span, AttributeHashKind::Outer(HashToken::new(span)))
    }

    /// Creates the `doc-comment` [AttributeDecl] for a single line of an inner comment. E.g.:
    /// ```ignore
    /// //! This is an inner comment.
    /// ```
    /// The `span` is the overall span: `//! This is an inner comment.`.
    /// The `content_span` is the span of the content, without the leading `//!`: ` This is an inner comment.`.
    pub fn new_inner_doc_comment(span: Span, content_span: Span) -> Self {
        Self::new_doc_comment(span.clone(), content_span, AttributeHashKind::Inner(HashBangToken::new(span)))
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
        self.attribute.inner.value_separator_pairs.is_empty() &&
        self.attribute.inner.final_value_opt.as_ref().is_some_and(|attr| attr.is_doc_comment())
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
