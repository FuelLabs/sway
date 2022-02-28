use crate::priv_prelude::*;

macro_rules! define_brackets (
    ($ty_name:ident, $delimiter:ident, $error:literal) => {
        #[derive(Clone, Debug)]
        pub struct $ty_name<T> {
            inner: T,
        }

        impl<T> $ty_name<T> {
            pub fn new<'a>(inner: T, _consumed: ParserConsumed<'a>) -> $ty_name<T> {
                $ty_name {
                    inner,
                }
            }

            pub fn get(&self) -> &T {
                &self.inner
            }

            pub fn into_inner(self) -> T {
                self.inner
            }
        }

        impl<T> $ty_name<T>
        where
            T: ParseToEnd,
        {
            pub fn try_parse(parser: &mut Parser) -> ParseResult<Option<$ty_name<T>>> {
                match parser.enter_delimited(Delimiter::$delimiter) {
                    Some(parser) => {
                        let (inner, _consumed) = parser.parse_to_end()?;
                        Ok(Some($ty_name { inner }))
                    },
                    None => Ok(None),
                }
            }
        }

        impl<T> $ty_name<T>
        where
            T: Parse,
        {
            pub fn parse_all_inner(
                parser: &mut Parser,
                on_error: impl FnOnce(Parser) -> ErrorEmitted,
            ) -> ParseResult<$ty_name<T>> {
                match parser.enter_delimited(Delimiter::$delimiter) {
                    Some(mut parser) => {
                        let inner = parser.parse()?;
                        if !parser.is_empty() {
                            return Err(on_error(parser))
                        }
                        Ok($ty_name { inner })
                    },
                    None => Err(parser.emit_error($error)),
                }
            }
        }

        impl<T> $ty_name<T>
        where
            T: Parse,
        {
            pub fn try_parse_all_inner(
                parser: &mut Parser,
                on_error: impl FnOnce(Parser) -> ErrorEmitted,
            ) -> ParseResult<Option<$ty_name<T>>> {
                match parser.enter_delimited(Delimiter::$delimiter) {
                    Some(mut parser) => {
                        let inner = parser.parse()?;
                        if !parser.is_empty() {
                            return Err(on_error(parser))
                        }
                        Ok(Some($ty_name { inner }))
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
                    Some(parser) => {
                        let (inner, _consumed) = parser.parse_to_end()?;
                        Ok($ty_name { inner })
                    },
                    None => Err(parser.emit_error($error)),
                }
            }
        }
    };
);

define_brackets!(Braces, Brace, "expected an opening brace");
define_brackets!(Parens, Parenthesis, "expected opening parenthesis");
define_brackets!(SquareBrackets, Bracket, "expected an opening square bracket");

#[derive(Clone, Debug)]
pub struct AngleBrackets<T> {
    pub less_than_token: LessThanToken,
    pub inner: T,
    pub greater_than_token: GreaterThanToken,
}

