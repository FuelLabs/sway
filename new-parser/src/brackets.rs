use crate::priv_prelude::*;

macro_rules! define_brackets (
    (
        $ty_name:ident,
        $err_name:ident,
        $fatal_err_name:ident,
        $fn_name:ident,
        $ty_open:ident,
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

        #[derive(Clone)]
        pub struct $err_name {
            pub position: usize,
        }

        #[derive(Clone)]
        pub enum $fatal_err_name<E, R> {
            Inner(E),
            InnerFatal(R),
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

        pub fn $fn_name<T, E, R, P>(parser: P)
            -> impl Parser<
                Output = $ty_name<T>,
                Error = $err_name,
                FatalError = $fatal_err_name<E, R>,
            > + Clone
        where
            P: Parser<Output = T, Error = E, FatalError = R> + Clone,
        {
            $fn_open()
            .map_err(|$err_open { position }| $err_name { position })
            .then(
                parser
                .map_err($fatal_err_name::Inner)
                .map_fatal_err($fatal_err_name::InnerFatal)
                .then(
                    $fn_close()
                    .map_err(|$err_close { position }| $fatal_err_name::$name_close { position })
                )
                .fatal()
            )
            .map(|(open_token, (inner, close_token))| $ty_name { open_token, inner, close_token })
        }
    );
);

define_brackets!(
    Parens, ParensError, ParensFatalError, parens,
    OpenParenToken, ExpectedOpenParenTokenError, open_paren_token,
    CloseParenToken, ExpectedCloseParen, ExpectedCloseParenTokenError, close_paren_token,
);
define_brackets!(
    SquareBrackets, SquareBracketsError, SquareBracketsFatalError, square_brackets,
    OpenSquareBracketToken, ExpectedOpenSquareBracketTokenError, open_square_bracket_token,
    CloseSquareBracketToken, ExpectedClosingSquareBracket, ExpectedCloseSquareBracketTokenError, close_square_bracket_token,
);
define_brackets!(
    Braces, BracesError, BracesFatalError, braces,
    OpenBraceToken, ExpectedOpenBraceTokenError, open_brace_token,
    CloseBraceToken, ExpectedCloseBrace, ExpectedCloseBraceTokenError, close_brace_token,
);
define_brackets!(
    AngleBrackets, AngleBracketsError, AngleBracketsFatalError, angle_brackets,
    LessThanToken, ExpectedLessThanTokenError, less_than_token,
    GreaterThanToken, ExpectedCloseAngleBracket, ExpectedGreaterThanTokenError, greater_than_token,
);
