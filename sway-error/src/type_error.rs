use sway_types::{Span, Spanned};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeError {
    #[error(
        "Mismatched types.\n\
         expected: {expected}\n\
         found:    {received}.\n\
         {help}",
         help = if !help_text.is_empty() { format!("help: {}", help_text) } else { String::new() }
    )]
    MismatchedType {
        expected: String,
        received: String,
        help_text: String,
        span: Span,
    },
    #[error("This type is not known. Try annotating it with a type annotation.")]
    UnknownType { span: Span },
    #[error(
        "The pattern for this match expression arm has a mismatched type.\n\
         expected: {expected}\n\
         found:    {received}.\n\
         "
    )]
    MatchArmScrutineeWrongType {
        expected: String,
        received: String,
        span: Span,
    },
}

impl Spanned for TypeError {
    fn span(&self) -> Span {
        use TypeError::*;
        match self {
            MismatchedType { span, .. } => span.clone(),
            UnknownType { span } => span.clone(),
            MatchArmScrutineeWrongType { span, .. } => span.clone(),
        }
    }
}
