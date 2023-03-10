use crate::priv_prelude::*;

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
    /// Should only ever be used for destructuring syntactic sugar since it does not have a literal span.
    pub fn new(inner: T) -> Self {
        Self {
            open_token: OpenParenthesisToken {
                span: Span::dummy(),
            },
            inner,
            close_token: CloseParenthesisToken {
                span: Span::dummy(),
            },
        }
    }
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
    /// Should only ever be used for destructuring syntactic sugar since it does not have a literal span.
    pub fn new(inner: T) -> Self {
        Self {
            open_token: OpenSquareBracketToken {
                span: Span::dummy(),
            },
            inner,
            close_token: CloseSquareBracketToken {
                span: Span::dummy(),
            },
        }
    }
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
