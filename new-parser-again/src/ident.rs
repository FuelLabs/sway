use crate::priv_prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ident {
    pub(crate) span: Span,
}

impl Ident {
    pub fn as_str(&self) -> &str {
        self.span.as_str()
    }

    pub fn span(&self) -> Span {
        self.span.clone()
    }
}

impl Peek for Ident {
    fn peek(peeker: Peeker<'_>) -> Option<Ident> {
        peeker.peek_ident().ok().map(Ident::clone)
    }
}

impl Parse for Ident {
    fn parse(parser: &mut Parser) -> ParseResult<Ident> {
        match parser.take() {
            Some(ident) => Ok(ident),
            None => Err(parser.emit_error("expected an identifier")),
        }
    }
}

