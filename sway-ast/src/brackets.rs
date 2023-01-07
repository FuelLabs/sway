use crate::priv_prelude::*;

// macro_rules! define_brackets (
//     ($ty_name:ident, $open:O, $close:C) => {
//         #[derive(Clone, Debug)]
//         pub struct $ty_name<T> {
//             pub open_token: $open,
//             pub inner: T,
//             pub close_token: $close,
//         }

//         impl<T> $ty_name<T> {
//             pub fn new<'a>(inner: T) -> $ty_name<T> {
//                 $ty_name {
//                     inner,
//                 }
//             }

//             pub fn get(&self) -> &T {
//                 &self.inner
//             }

//             pub fn into_inner(self) -> T {
//                 self.inner
//             }
//         }

//         impl<T> Spanned for $ty_name<T> {
//             fn span(&self) -> Span {
//                 Span::join(
//                     self.open_token.span(),
//                     self.close_token.span(),
//                 )
//             }
//         }
//     };
// );

#[derive(Clone, Debug)]
pub struct Braces<T> {
    pub open_token: OpenCurlyBraceToken,
    pub inner: T,
    pub close_token: CloseCurlyBraceToken,
}
impl<T> Braces<T> {
    pub fn get(&self) -> &T {
        &self.inner
    }
    pub fn into_inner(self) -> T {
        self.inner
    }
}
impl<T> Spanned for Braces<T> {
    fn span(&self) -> Span {
        Span::join(self.open_token.span(), self.close_token.span())
    }
}
#[derive(Clone, Debug)]
pub struct Parens<T> {
    pub open_token: OpenParenthesisToken,
    pub inner: T,
    pub close_token: CloseParenthesisToken,
}
impl<T> Parens<T> {
    pub fn get(&self) -> &T {
        &self.inner
    }
    pub fn into_inner(self) -> T {
        self.inner
    }
}
impl<T> Spanned for Parens<T> {
    fn span(&self) -> Span {
        Span::join(self.open_token.span(), self.close_token.span())
    }
}
#[derive(Clone, Debug)]
pub struct SquareBrackets<T> {
    pub open_token: OpenSquareBracketToken,
    pub inner: T,
    pub close_token: CloseSquareBracketToken,
}
impl<T> SquareBrackets<T> {
    pub fn get(&self) -> &T {
        &self.inner
    }
    pub fn into_inner(self) -> T {
        self.inner
    }
}
impl<T> Spanned for SquareBrackets<T> {
    fn span(&self) -> Span {
        Span::join(self.open_token.span(), self.close_token.span())
    }
}

#[derive(Clone, Debug)]
pub struct AngleBrackets<T> {
    pub open_angle_bracket_token: OpenAngleBracketToken,
    pub inner: T,
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
