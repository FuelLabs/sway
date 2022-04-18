use crate::priv_prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct LitString {
    pub span: Span,
    pub parsed: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct LitChar {
    pub span: Span,
    pub parsed: char,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct LitInt {
    pub span: Span,
    pub parsed: BigUint,
    pub ty_opt: Option<(LitIntType, Span)>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum LitIntType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum Literal {
    String(LitString),
    Char(LitChar),
    Int(LitInt),
}

impl Peek for Literal {
    fn peek(peeker: Peeker<'_>) -> Option<Literal> {
        peeker.peek_literal().ok().map(Literal::clone)
    }
}

impl Parse for Literal {
    fn parse(parser: &mut Parser) -> ParseResult<Literal> {
        match parser.take() {
            Some(literal) => Ok(literal),
            None => Err(parser.emit_error(ParseErrorKind::ExpectedLiteral)),
        }
    }
}

impl LitString {
    pub fn span(&self) -> Span {
        self.span.clone()
    }
}

impl LitChar {
    pub fn span(&self) -> Span {
        self.span.clone()
    }
}

impl LitInt {
    pub fn span(&self) -> Span {
        match &self.ty_opt {
            Some((_lit_int_ty, span)) => Span::join(self.span.clone(), span.clone()),
            None => self.span.clone(),
        }
    }
}

impl Literal {
    pub fn span(&self) -> Span {
        match self {
            Literal::String(lit_string) => lit_string.span(),
            Literal::Char(lit_char) => lit_char.span(),
            Literal::Int(lit_int) => lit_int.span(),
        }
    }
}
