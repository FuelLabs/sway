use crate::priv_prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LitString {
    pub span: Span,
    pub parsed: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LitChar {
    pub span: Span,
    pub parsed: char,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LitInt {
    pub span: Span,
    pub parsed: BigUint,
    pub ty_opt: Option<(LitIntType, Span)>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
            None => Err(parser.emit_error("expected a literal")),
        }
    }
}

