use crate::priv_prelude::*;

macro_rules! define_brackets (
    (
        $ty_name:ident,
        $err_name:ident,
        $fn_name:ident,
        $ty_open:ident,
        $name_open:ident,
        $err_open:ident,
        $fn_open:ident,
        $ty_close:ident,
        $name_close:ident,
        $err_close:ident,
        $fn_close:ident,
    ) => (
        #[derive(Clone, Debug)]
        pub struct $ty_name<T> {
            open_token: $ty_open,
            inner: T,
            close_token: $ty_close,
        }

        pub enum $err_name<E> {
            $name_open { position: usize },
            Parser(E),
            $name_close { position: usize },
        }

        impl<T> $ty_name<T> {
            pub fn inner(&self) -> &T {
                &self.inner
            }

            pub fn map<F, U>(self, func: F) -> $ty_name<U>
            where
                F: FnOnce(T) -> U,
            {
                let $ty_name { open_token, inner, close_token } = self;
                let inner = func(inner);
                $ty_name { open_token, inner, close_token }
            }
        }

        impl<T> Spanned for $ty_name<T> {
            fn span(&self) -> Span {
                Span::join(self.open_token.span(), self.close_token.span())
            }
        }

        impl<T> std::ops::Deref for $ty_name<T> {
            type Target = T;
            
            fn deref(&self) -> &T {
                self.inner()
            }
        }

        pub fn $fn_name<T, E, P>(parser: P) -> impl Parser<Output = $ty_name<T>, Error = $err_name<E>> + Clone
        where
            P: Parser<Output = T, Error = E> + Clone,
        {
            $fn_open()
            .map_err(|$err_open { position }| $err_name::$name_open { position })
            .then(
                parser
                .map_err($err_name::Parser)
            )
            .then(
                $fn_close()
                .map_err(|$err_close { position }| $err_name::$name_close { position })
            )
            .map(|((open_token, inner), close_token)| $ty_name { open_token, inner, close_token })
        }
    );
);

define_brackets!(
    Parens, ParensError, parens,
    OpenParenToken, ExpectedOpenParen, ExpectedOpenParenTokenError, open_paren_token,
    CloseParenToken, ExpectedCloseParen, ExpectedCloseParenTokenError, close_paren_token,
);
define_brackets!(
    SquareBrackets, SquareBracketsError, square_brackets,
    OpenSquareBracketToken, ExpectedOpenSquareBracket, ExpectedOpenSquareBracketTokenError, open_square_bracket_token,
    CloseSquareBracketToken, ExpectedClosingSquareBracket, ExpectedCloseSquareBracketTokenError, close_square_bracket_token,
);
define_brackets!(
    Braces, BracesError, braces,
    OpenBraceToken, ExpectedOpenBrace, ExpectedOpenBraceTokenError, open_brace_token,
    CloseBraceToken, ExpectedCloseBrace, ExpectedCloseBraceTokenError, close_brace_token,
);
define_brackets!(
    AngleBrackets, AngleBracketsError, angle_brackets,
    LessThanToken, ExpectedOpenAngleBracket, ExpectedLessThanTokenError, less_than_token,
    GreaterThanToken, ExpectedCloseAngleBracket, ExpectedGreaterThanTokenError, greater_than_token,
);
