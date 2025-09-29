use crate::{Parse, ParseResult, ParseToEnd, Parser};

use sway_ast::brackets::{Braces, Parens, SquareBrackets};
use sway_ast::keywords::{CloseAngleBracketToken, OpenAngleBracketToken};
use sway_error::handler::ErrorEmitted;
use sway_error::parser_error::ParseErrorKind;
use sway_types::{ast::Delimiter, Span, Spanned};

pub trait ParseBracket<T>: Sized {
    fn try_parse(parser: &mut Parser) -> ParseResult<Option<Self>>
    where
        T: ParseToEnd;

    fn parse_all_inner(
        parser: &mut Parser,
        on_error: impl FnOnce(Parser) -> ErrorEmitted,
    ) -> ParseResult<Self>
    where
        T: Parse;

    fn try_parse_all_inner(
        parser: &mut Parser,
        on_error: impl FnOnce(Parser) -> ErrorEmitted,
    ) -> ParseResult<Option<Self>>
    where
        T: Parse;
}

macro_rules! impl_brackets (
    ($ty_name:ident, $delimiter:ident, $error:ident) => {
        impl<T> ParseBracket<T> for $ty_name<T> {
            fn try_parse(parser: &mut Parser) -> ParseResult<Option<$ty_name<T>>>
            where
                T: ParseToEnd
            {
                match parser.enter_delimited(Delimiter::$delimiter) {
                    Some((parser, span)) => {
                        let (inner, _consumed) = parser.parse_to_end()?;
                        Ok(Some($ty_name { inner, span }))
                    },
                    None => Ok(None),
                }
            }

            fn parse_all_inner(
                parser: &mut Parser,
                on_error: impl FnOnce(Parser) -> ErrorEmitted,
            ) -> ParseResult<$ty_name<T>>
            where
                T: Parse
            {
                match parser.enter_delimited(Delimiter::$delimiter) {
                    Some((mut parser, span)) => {
                        let inner = parser.parse()?;
                        if !parser.is_empty() {
                            return Err(on_error(parser))
                        }
                        Ok($ty_name { inner, span })
                    },
                    None => Err(parser.emit_error(ParseErrorKind::$error)),
                }
            }

            fn try_parse_all_inner(
                parser: &mut Parser,
                on_error: impl FnOnce(Parser) -> ErrorEmitted,
            ) -> ParseResult<Option<$ty_name<T>>>
            where
                T: Parse
            {
                match parser.enter_delimited(Delimiter::$delimiter) {
                    Some((mut parser, span)) => {
                        let inner = parser.parse()?;
                        if !parser.is_empty() {
                            return Err(on_error(parser))
                        }
                        Ok(Some($ty_name { inner, span }))
                    },
                    None => Ok(None),
                }
            }
        }

        impl<T> Parse for $ty_name<T>
        where
            T: ParseToEnd,
        {
            fn parse(parser: &mut Parser) -> ParseResult<$ty_name<T>> {
                match parser.enter_delimited(Delimiter::$delimiter) {
                    Some((parser, span)) => {
                        let (inner, _consumed) = parser.parse_to_end()?;
                        Ok($ty_name { inner, span })
                    },
                    None => Err(parser.emit_error(ParseErrorKind::$error)),
                }
            }
        }
    };
);

impl_brackets!(Braces, Brace, ExpectedOpenBrace);
impl_brackets!(Parens, Parenthesis, ExpectedOpenParen);
impl_brackets!(SquareBrackets, Bracket, ExpectedOpenBracket);

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct AngleBrackets<T> {
    pub open_angle_bracket_token: OpenAngleBracketToken,
    #[allow(unused)]
    pub inner: T,
    pub close_angle_bracket_token: CloseAngleBracketToken,
}

impl<T> Spanned for AngleBrackets<T> {
    fn span(&self) -> Span {
        Span::join(
            self.open_angle_bracket_token.span(),
            &self.close_angle_bracket_token.span(),
        )
    }
}
