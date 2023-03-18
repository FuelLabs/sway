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
                if let Some(mut parser)
                    = parser.enter_delimited($ty_name::<T>::as_opening_delimiter())
                {
                    if parser.peek::<$open_token>().is_some() {
                        let open_token = parser.parse()?;
                        let inner = parser.parse()?;
                        if parser.peek::<$close_token>().is_some() {
                            return Ok(Some(
                                $ty_name {
                                    open_token,
                                    inner,
                                    close_token: parser.parse()?,
                                })
                            )
                        }
                        return Ok(None)
                    }
                    return Ok(None)
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

                if let Some(mut parser)
                    = parser.enter_delimited($ty_name::<T>::as_opening_delimiter())
                {
                    if parser.peek::<$open_token>().is_some() {
                        let open_token = parser.parse()?;
                        let inner = parser.parse()?;
                        if parser.peek::<$close_token>().is_some() {
                            let close_token = parser.parse()?;
                            if !parser.is_empty() {
                                return Err(on_error(parser))
                            }
                            return Ok(
                                $ty_name {
                                    open_token,
                                    inner,
                                    close_token,
                                }
                            )
                        }
                        return Err(parser.emit_error(ParseErrorKind::ExpectedClosingDelimiter { kinds: vec![$close_kind] }))
                    }
                    return Err(parser.emit_error(ParseErrorKind::ExpectedOpeningDelimiter { kinds: vec![$open_kind] }))
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
                if let Some(mut parser)
                    = parser.enter_delimited($ty_name::<T>::as_opening_delimiter())
                {
                    if parser.peek::<$open_token>().is_some() {
                        let open_token = parser.parse()?;
                        let inner = parser.parse()?;
                        if parser.peek::<$close_token>().is_some() {
                            let close_token = parser.parse()?;
                            if !parser.is_empty() {
                                return Err(on_error(parser))
                            }
                            return Ok(Some(
                                $ty_name {
                                    open_token,
                                    inner,
                                    close_token,
                                })
                            )
                        }
                        return Ok(None)
                    }
                    return Ok(None)
                }
                Ok(None)
            }
        }

        impl<T> Parse for $ty_name<T>
        where
            T: Parse
        {
            fn parse(
                parser: &mut Parser,
            ) -> ParseResult<$ty_name<T>>
            {
                if let Some(mut parser)
                    = parser.enter_delimited($ty_name::<T>::as_opening_delimiter())
                {
                    if parser.peek::<$open_token>().is_some() {
                        let open_token = parser.parse()?;
                        let inner = parser.parse()?;
                        if parser.peek::<$close_token>().is_some() {
                            return Ok(
                                $ty_name {
                                    open_token,
                                    inner,
                                    close_token: parser.parse()?,
                                })
                        }
                        return Err(parser.emit_error(ParseErrorKind::ExpectedClosingDelimiter { kinds: vec![$close_kind] }))
                    }
                    return Err(parser.emit_error(ParseErrorKind::ExpectedOpeningDelimiter { kinds: vec![$open_kind] }))
                }
                Err(parser.emit_error(ParseErrorKind::ExpectedOpeningDelimiter { kinds: vec![$open_kind] }))
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
