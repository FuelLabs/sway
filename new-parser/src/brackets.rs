use crate::priv_prelude::*;

macro_rules! define_brackets (
    ($ty_name:ident, $fn_name:ident, $ty_open:ident, $fn_open:ident, $ty_close:ident, $fn_close:ident,) => (
        pub struct $ty_name<T> {
            open_token: $ty_open,
            inner: T,
            close_token: $ty_close,
        }

        impl<T> $ty_name<T> {
            pub fn inner(&self) -> &T {
                &self.inner
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

        pub fn $fn_name<T, P>(parser: P) -> impl Parser<Output = $ty_name<T>> + Clone
        where
            P: Parser<Output = T> + Clone,
        {
            $fn_open()
            .then(parser)
            .then($fn_close())
            .map(|((open_token, inner), close_token)| $ty_name { open_token, inner, close_token })
        }
    );
);

define_brackets!(
    Parens, parens,
    OpenParenToken, open_paren_token,
    CloseParenToken, close_paren_token,
);
define_brackets!(
    SquareBrackets, square_brackets,
    OpenSquareBracketToken, open_square_bracket_token,
    CloseSquareBracketToken, close_square_bracket_token,
);
define_brackets!(
    Braces, braces,
    OpenBraceToken, open_brace_token,
    CloseBraceToken, close_brace_token,
);
define_brackets!(
    AngleBrackets, angle_brackets,
    LessThanToken, less_than_token,
    GreaterThanToken, greater_than_token,
);

