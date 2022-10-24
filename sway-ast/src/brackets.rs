use crate::priv_prelude::*;

macro_rules! define_brackets (
    ($ty_name:ident) => {
        #[derive(Clone, Debug)]
        pub struct $ty_name<T> {
            pub inner: T,
            pub span: Span,
        }

        impl<T> $ty_name<T> {
            pub fn new<'a>(inner: T, span: Span) -> $ty_name<T> {
                $ty_name {
                    inner,
                    span,
                }
            }

            pub fn get(&self) -> &T {
                &self.inner
            }

            pub fn into_inner(self) -> T {
                self.inner
            }
        }

        impl<T> Spanned for $ty_name<T> {
            fn span(&self) -> Span {
                self.span.clone()
            }
        }
    };
);

define_brackets!(Braces);
define_brackets!(Parens);
define_brackets!(SquareBrackets);

#[derive(Clone, Debug)]
pub struct AngleBrackets<T> {
    pub open_angle_bracket_token:  OpenAngleBracketToken,
    pub inner:                     T,
    pub close_angle_bracket_token: CloseAngleBracketToken,
}

impl<T> AngleBrackets<T> {
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> Spanned for AngleBrackets<T> {
    fn span(&self) -> Span {
        Span::join(
            self.open_angle_bracket_token.span(),
            self.close_angle_bracket_token.span(),
        )
    }
}
