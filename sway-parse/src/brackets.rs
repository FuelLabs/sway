use crate::{Parse, ParseResult, Parser};

use sway_ast::brackets::{Braces, Parens, SquareBrackets};
use sway_ast::keywords::{
    CloseCurlyBraceToken, CloseParenthesisToken, CloseSquareBracketToken, OpenCurlyBraceToken,
    OpenParenthesisToken, OpenSquareBracketToken,
};
use sway_ast::token::{
    ClosingDelimiter::{
        CurlyBrace as ClosingCurlyBrace, Parenthesis as ClosingParenthesis,
        SquareBracket as ClosingSquareBracket,
    },
    OpeningDelimiter::{
        CurlyBrace as OpeningCurlyBrace, Parenthesis as OpeningParenthesis,
        SquareBracket as OpeningSquareBracket,
    },
};
use sway_error::handler::ErrorEmitted;
use sway_error::parser_error::ParseErrorKind;

pub trait ParseBracket<T>: Sized {
    fn try_parse(parser: &mut Parser) -> ParseResult<Option<Self>>
    where
        T: Parse;

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
    (
        $ty_name:ident,
        $open_token:ident,
        $close_token:ident,
        $open_kind:ident,
        $close_kind:ident
    ) => {
        impl<T> ParseBracket<T> for $ty_name<T> {
            fn try_parse(parser: &mut Parser) -> ParseResult<Option<$ty_name<T>>>
            where
                T: Parse
            {
                if let Some(open_token) = parser.peek::<$open_token>() {
                    let inner = parser.parse()?;
                    match parser.peek::<$close_token>() {
                       Some(close_token) => Ok(Some(
                            $ty_name {
                                open_token,
                                inner,
                                close_token,
                            })
                        ),
                        None => Ok(None)
                    };
                }
                Ok(None)
            }

            fn parse_all_inner(
                parser: &mut Parser,
                on_error: impl FnOnce(Parser) -> ErrorEmitted,
            ) -> ParseResult<$ty_name<T>>
            where
                T: Parse
            {
                if let Some(open_token) = parser.peek::<$open_token>() {
                    let inner = parser.parse()?;
                    match parser.peek::<$close_token>() {
                       Some(close_token) => {
                            if !parser.is_empty() {
                                return Err(on_error(*parser))
                            }
                            Ok(Some(
                                $ty_name {
                                    open_token,
                                    inner,
                                    close_token,
                                })
                            )
                        },
                        None => Err(parser.emit_error(ParseErrorKind::ExpectedClosingDelimiter { kinds: vec![$close_kind] }))
                    };
                }
                Err(parser.emit_error(ParseErrorKind::ExpectedOpeningDelimiter { kinds: vec![$open_kind] }))
            }

            fn try_parse_all_inner(
                parser: &mut Parser,
                on_error: impl FnOnce(Parser) -> ErrorEmitted,
            ) -> ParseResult<Option<$ty_name<T>>>
            where
                T: Parse
            {
                if let Some(open_token) = parser.peek::<$open_token>() {
                    let inner = parser.parse()?;
                    match parser.peek::<$close_token>() {
                       Some(close_token) => {
                            if !parser.is_empty() {
                                return Err(on_error(*parser))
                            }
                            Ok(Some(
                                $ty_name {
                                    open_token,
                                    inner,
                                    close_token,
                                })
                            )
                        },
                        None => Ok(None)
                    };
                }
                Ok(None)
            }

        }
    };
);

impl_brackets!(
    Braces,
    OpenCurlyBraceToken,
    CloseCurlyBraceToken,
    OpeningCurlyBrace,
    ClosingCurlyBrace
);
impl_brackets!(
    Parens,
    OpenParenthesisToken,
    CloseParenthesisToken,
    OpeningParenthesis,
    ClosingParenthesis
);
impl_brackets!(
    SquareBrackets,
    OpenSquareBracketToken,
    CloseSquareBracketToken,
    OpeningSquareBracket,
    ClosingSquareBracket
);
