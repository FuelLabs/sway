use crate::priv_prelude::*;

#[derive(Clone, Debug, Serialize)]
pub struct Annotated<T> {
    pub attribute_list: Vec<AttributeDecl>,
    pub value: T,
}

// Attributes can have any number of arguments:
//
//    #[attribute]
//    #[attribute()]
//    #[attribute(value)]
//    #[attribute(value0, value1, value2)]

#[derive(Clone, Debug, Serialize)]
pub struct AttributeDecl {
    pub hash_kind: AttributeHashKind,
    pub attribute: SquareBrackets<Punctuated<Attribute, CommaToken>>,
}

impl Spanned for AttributeDecl {
    fn span(&self) -> Span {
        let hash_span = match &self.hash_kind {
            AttributeHashKind::Inner(hash_bang_token) => hash_bang_token.span(),
            AttributeHashKind::Outer(hash_token) => hash_token.span(),
        };
        Span::join(hash_span, self.attribute.span())
    }
}

/// Denotes the target direction of an [AttributeDecl] and
/// the hash token kind associated.
///
/// Example:
/// ```sway
/// // outer (after), written as `///`
/// #[doc("a Sway struct")]
/// struct Foo {}
///
/// // inner (before), written as `//!`
/// enum Bar {}
/// #![doc("a Sway enum")]
/// ```
#[derive(Clone, Debug, Serialize)]
pub enum AttributeHashKind {
    /// Inner specifies that the attribute belongs to
    /// the item before it.
    Inner(HashBangToken),
    /// Outer specifies that the attribute belongs to
    /// the item after it.
    Outer(HashToken),
}

#[derive(Clone, Debug, Serialize)]
pub struct Attribute {
    pub name: Ident,
    pub args: Option<Parens<Punctuated<Ident, CommaToken>>>,
}

impl Spanned for Attribute {
    fn span(&self) -> Span {
        self.args
            .as_ref()
            .map(|args| Span::join(self.name.span(), args.span()))
            .unwrap_or_else(|| self.name.span())
    }
}
