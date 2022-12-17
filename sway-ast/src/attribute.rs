use crate::priv_prelude::*;

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct AttributeDecl {
    pub hash_token: HashToken,
    pub attribute: SquareBrackets<Punctuated<Attribute, CommaToken>>,
}

impl Spanned for AttributeDecl {
    fn span(&self) -> Span {
        Span::join(self.hash_token.span(), self.attribute.span())
    }
}

#[derive(Clone, Debug)]
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
