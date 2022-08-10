use crate::keywords::RESERVED_KEYWORDS;
use crate::{ParseErrorKind, ParseResult, Parser, ParserConsumed, Peeker};

use sway_ast::token::Delimiter;
use sway_ast::Intrinsic;
use sway_types::{Ident, Spanned};

pub trait Parse {
    fn parse(parser: &mut Parser) -> ParseResult<Self>
    where
        Self: Sized;
}

pub trait Peek {
    fn peek(peeker: Peeker<'_>) -> Option<Self>
    where
        Self: Sized;
}

pub trait ParseToEnd {
    fn parse_to_end<'a, 'e>(parser: Parser<'a, 'e>) -> ParseResult<(Self, ParserConsumed<'a>)>
    where
        Self: Sized;
}

impl<T> Parse for Box<T>
where
    T: Parse,
{
    fn parse(parser: &mut Parser) -> ParseResult<Box<T>> {
        let value = parser.parse()?;
        Ok(Box::new(value))
    }
}

macro_rules! impl_tuple (
    ($($name:ident,)*) => {
        impl<$($name,)*> Parse for ($($name,)*)
        where
            $($name: Parse,)*
        {
            fn parse(parser: &mut Parser) -> ParseResult<($($name,)*)> {
                #[allow(unused)]
                let parser = parser;
                $(
                    #[allow(non_snake_case)]
                    let $name = parser.parse()?;
                )*
                Ok(($($name,)*))
            }
        }

        impl<$($name,)*> Peek for ($($name,)*)
        where
            $($name: Peek,)*
        {
            fn peek(peeker: Peeker<'_>) -> Option<Self> {
                #![allow(unused_assignments, unused, non_snake_case)]

                let mut tokens = peeker.token_trees;
                $(
                    let ($name, fewer_tokens) = Peeker::with::<$name>(tokens)?;
                    tokens = fewer_tokens;

                )*
                Some(($($name,)*))
            }
        }
    };
);

impl_tuple!();
impl_tuple!(T0,);
impl_tuple!(T0, T1,);
impl_tuple!(T0, T1, T2,);
impl_tuple!(T0, T1, T2, T3,);
impl_tuple!(T0, T1, T2, T3, T4,);
impl_tuple!(T0, T1, T2, T3, T4, T5,);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6,);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7,);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8,);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9,);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10,);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11,);

impl<T> ParseToEnd for Vec<T>
where
    T: Parse,
{
    fn parse_to_end<'a, 'e>(
        mut parser: Parser<'a, 'e>,
    ) -> ParseResult<(Vec<T>, ParserConsumed<'a>)> {
        let mut ret = Vec::new();
        loop {
            if let Some(consumed) = parser.check_empty() {
                return Ok((ret, consumed));
            }
            let value = parser.parse()?;
            ret.push(value);
        }
    }
}

impl Peek for Ident {
    fn peek(peeker: Peeker<'_>) -> Option<Ident> {
        peeker.peek_ident().ok().map(Ident::clone)
    }
}

impl Parse for Ident {
    fn parse(parser: &mut Parser) -> ParseResult<Ident> {
        match parser.take::<Ident>() {
            Some(ident) => {
                let ident_str = ident.as_str();
                if ident_str.starts_with("__") && Intrinsic::try_from_str(ident_str).is_none() {
                    return Err(parser.emit_error_with_span(
                        ParseErrorKind::InvalidDoubleUnderscore,
                        ident.span(),
                    ));
                }

                if !ident.is_raw_ident() && RESERVED_KEYWORDS.contains(ident_str) {
                    return Err(parser.emit_error_with_span(
                        ParseErrorKind::ReservedKeywordIdentifier,
                        ident.span(),
                    ));
                }

                Ok(ident)
            }
            None => Err(parser.emit_error(ParseErrorKind::ExpectedIdent)),
        }
    }
}

impl Peek for Delimiter {
    fn peek(peeker: Peeker<'_>) -> Option<Delimiter> {
        peeker.peek_delimiter().ok()
    }
}
