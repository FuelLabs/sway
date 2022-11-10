use crate::{Parse, ParseResult, Parser, Peek, Peeker};

use sway_ast::Literal;
use sway_error::parser_error::ParseErrorKind;

impl Peek for Literal {
    fn peek(peeker: Peeker<'_>) -> Option<Literal> {
        peeker.peek_literal().ok().cloned()
    }
}

impl Parse for Literal {
    fn parse(parser: &mut Parser) -> ParseResult<Literal> {
        parser
            .take()
            .ok_or_else(|| parser.emit_error(ParseErrorKind::ExpectedLiteral))
    }
}
