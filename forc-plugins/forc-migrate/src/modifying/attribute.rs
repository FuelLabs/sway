use sway_ast::{
    attribute::{Attribute, AttributeArg, CFG_ATTRIBUTE_NAME, DOC_COMMENT_ATTRIBUTE_NAME},
    brackets::SquareBrackets,
    keywords::{HashBangToken, HashToken, Token},
    AttributeDecl, Literal, Parens, Punctuated,
};
use sway_types::{Ident, Span, Spanned};

use crate::assert_insert_span;

use super::{Modifier, New};

#[allow(dead_code)]
impl Modifier<'_, Attribute> {
    pub(crate) fn set_name<S: AsRef<str> + ?Sized>(&mut self, name: &S) -> &mut Self {
        // We preserve the current span of the name.
        let insert_span = self.element.name.span();
        self.element.name = Ident::new_with_override(name.as_ref().into(), insert_span);

        self
    }
}

#[allow(dead_code)]
impl New {
    /// Creates an [Attribute] with a single [AttributeArg]. E.g. `attribute_name(arg_name = value)` or `attribute_name(arg_name)`.
    pub(crate) fn attribute_with_arg<S: AsRef<str> + ?Sized>(
        insert_span: Span,
        attribute_name: &S,
        arg_name: &S,
        value: Option<Literal>,
    ) -> Attribute {
        assert_insert_span!(insert_span);

        Attribute {
            name: Ident::new_with_override(attribute_name.as_ref().into(), insert_span.clone()),
            args: Some(Parens {
                inner: Punctuated {
                    value_separator_pairs: vec![],
                    final_value_opt: Some(Box::new(AttributeArg {
                        name: Ident::new_with_override(
                            arg_name.as_ref().into(),
                            insert_span.clone(),
                        ),
                        value,
                    })),
                },
                span: insert_span,
            }),
        }
    }

    /// Creates an [AttributeDecl] with a single [Attribute] that has a single [AttributeArg]. E.g. `#[attribute_name(arg_name = value)]` or `#[attribute_name(arg_name)]`.
    pub(crate) fn attribute_decl_with_arg<S: AsRef<str> + ?Sized>(
        insert_span: Span,
        attribute_name: &S,
        arg_name: &S,
        value: Option<Literal>,
    ) -> AttributeDecl {
        assert_insert_span!(insert_span);

        AttributeDecl {
            hash_kind: sway_ast::attribute::AttributeHashKind::Outer(HashToken::new(
                insert_span.clone(),
            )),
            attribute: SquareBrackets {
                inner: Punctuated {
                    value_separator_pairs: vec![],
                    final_value_opt: Some(Box::new(New::attribute_with_arg(
                        insert_span.clone(),
                        attribute_name,
                        arg_name,
                        value,
                    ))),
                },
                span: insert_span,
            },
        }
    }

    /// Creates an [AttributeDecl] representing a single `cfg` experimental attribute. E.g. `#[cfg(experimental_flag = value)]`.
    pub(crate) fn cfg_experimental_attribute_decl(
        insert_span: Span,
        feature_name: &str,
        value: bool,
    ) -> AttributeDecl {
        assert_insert_span!(insert_span);

        AttributeDecl {
            hash_kind: sway_ast::attribute::AttributeHashKind::Outer(HashToken::new(
                insert_span.clone(),
            )),
            attribute: SquareBrackets {
                inner: Punctuated {
                    value_separator_pairs: vec![],
                    final_value_opt: Some(Box::new(New::attribute_with_arg(
                        insert_span.clone(),
                        CFG_ATTRIBUTE_NAME,
                        &format!("experimental_{feature_name}"),
                        Some(New::literal_bool(insert_span.clone(), value)),
                    ))),
                },
                span: insert_span,
            },
        }
    }

    /// Creates a `doc-comment` [AttributeDecl] that defines a single doc-comment line.
    /// It automatically inserts the leading space.
    ///
    /// E.g., `comment` "This is a comment." will create an [AttributeDecl] that corresponds to:
    /// ```ignore
    /// //! This is a comment.
    /// ```
    pub(crate) fn doc_comment_attribute_decl<S: AsRef<str> + ?Sized>(
        insert_span: Span,
        comment: &S,
    ) -> AttributeDecl {
        assert_insert_span!(insert_span);

        AttributeDecl {
            hash_kind: sway_ast::attribute::AttributeHashKind::Inner(HashBangToken::new(
                insert_span.clone(),
            )),
            attribute: SquareBrackets {
                inner: Punctuated {
                    value_separator_pairs: vec![],
                    final_value_opt: Some(Box::new(New::attribute_with_arg(
                        insert_span.clone(),
                        DOC_COMMENT_ATTRIBUTE_NAME,
                        &format!(" {}", comment.as_ref()),
                        None,
                    ))),
                },
                span: insert_span,
            },
        }
    }

    /// Creates `doc-comment` [AttributeDecl]s that define multiple doc-comment lines.
    /// It automatically inserts the leading space into each line.
    ///
    /// E.g., `comments` \["This is a comment.", "This is the second line."\] will create
    /// [AttributeDecl]s that corresponds to:
    /// ```ignore
    /// //! This is a comment.
    /// //! This is the second line.
    /// ```
    pub(crate) fn doc_comments_attribute_decls<S: AsRef<str> + ?Sized>(
        insert_span: Span,
        comments: &[&S],
    ) -> Vec<AttributeDecl> {
        comments
            .iter()
            .map(|comment| New::doc_comment_attribute_decl(insert_span.clone(), comment))
            .collect()
    }
}
