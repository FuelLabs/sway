use pest::Span;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum CompileError<'sc> {
    #[error("Variable \"{var_name}\" does not exist in this scope.\"")]
    UnknownVariable { var_name: &'sc str, span: Span<'sc> },
    #[error("Function \"{name}\" does not exist in this scope.\"")]
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
    #[error(transparent)]
    TypeError(#[from] TypeError),
}

#[derive(Error, Debug)]
pub(crate) enum TypeError {
    #[error("Mismatched types: Expected type {expected} but received type {received}. Type {received} is not castable to type {expected}")]
    MismatchedType { expected: String, received: String },
}
