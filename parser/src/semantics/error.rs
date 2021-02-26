use crate::error::ParseError;
use pest::Span;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompileError<'sc> {
    #[error("Variable \"{var_name}\" does not exist in this scope.")]
    UnknownVariable { var_name: &'sc str, span: Span<'sc> },
    #[error("Function \"{name}\" does not exist in this scope.")]
    UnknownFunction { name: &'sc str, span: Span<'sc> },
    #[error("Identifier \"{name}\" was used as a variable, but it is actually a {what_it_is}.")]
    NotAVariable {
        name: &'sc str,
        span: Span<'sc>,
        what_it_is: &'static str,
    },
    #[error("Identifier \"{name}\" was called as if it was a function, but it is actually a {what_it_is}.")]
    NotAFunction {
        name: &'sc str,
        span: Span<'sc>,
        what_it_is: &'static str,
    },
    #[error("Internal compiler error: {0}\nPlease file an issue on the repository and include the code that triggered this error.")]
    Internal(&'static str, Span<'sc>),
    #[error("Type error: {0}")]
    TypeError(TypeError<'sc>),
    #[error("Parse error: {0}")]
    ParseError(ParseError<'sc>),
}

impl<'sc> std::convert::From<ParseError<'sc>> for CompileError<'sc> {
    fn from(other: ParseError<'sc>) -> CompileError<'sc> {
        CompileError::ParseError(other)
    }
}
impl<'sc> std::convert::From<TypeError<'sc>> for CompileError<'sc> {
    fn from(other: TypeError<'sc>) -> CompileError<'sc> {
        CompileError::TypeError(other)
    }
}

#[derive(Error, Debug)]
pub enum TypeError<'sc> {
    #[error("Mismatched types: Expected type {expected} but received type {received}. Type {received} is not castable to type {expected}.")]
    MismatchedType {
        expected: String,
        received: String,
        span: Span<'sc>,
    },
}

impl<'sc> TypeError<'sc> {
    pub(crate) fn span(&self) -> (usize, usize) {
        use TypeError::*;
        match self {
            MismatchedType { span, .. } => (span.start(), span.end()),
        }
    }
}

impl<'sc> CompileError<'sc> {
    pub fn to_friendly_error_string(&self) -> String {
        use CompileError::*;
        match self {
            ParseError(err) => err.to_friendly_error_string(),
            a => format!("{}", a),
        }
    }

    pub fn span(&self) -> (usize, usize) {
        use CompileError::*;
        match self {
            ParseError(err) => err.span(),
            UnknownVariable { span, .. } => (span.start(), span.end()),
            UnknownFunction { span, .. } => (span.start(), span.end()),
            NotAVariable { span, .. } => (span.start(), span.end()),
            NotAFunction { span, .. } => (span.start(), span.end()),
            Internal(_, span) => (span.start(), span.end()),
            TypeError(err) => err.span(),
        }
    }
}
