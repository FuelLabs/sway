use crate::priv_prelude::*;

#[derive(Debug, Clone)]
pub enum ParseError {
    ExpectedKeyword {
        word: &'static str,
        span: Span,
    },
    UnexpectedEof {
        span: Span,
    },
    ExpectedEof {
        span: Span,
    },
    ExpectedIdent {
        span: Span,
    },
    ExpectedWhitespace {
        span: Span,
    },
    ExpectedDigit {
        span: Span,
    },
    Or {
        error0: Box<ParseError>,
        error1: Box<ParseError>,
    },
    UnicodeEscapeOutOfRange {
        span: Span,
    },
    InvalidUnicodeEscapeChar {
        span: Span,
    },
    UnclosedMultilineComment {
        span: Span,
    },
    ExpectedNewline {
        span: Span,
    },
}

impl Spanned for ParseError {
    fn span(&self) -> Span {
        match self {
            ParseError::ExpectedKeyword { span, .. } => span.clone(),
            ParseError::UnexpectedEof { span } => span.clone(),
            ParseError::ExpectedEof { span } => span.clone(),
            ParseError::ExpectedIdent { span } => span.clone(),
            ParseError::ExpectedWhitespace { span } => span.clone(),
            ParseError::ExpectedDigit { span } => span.clone(),
            ParseError::Or { error0, error1 } => {
                Span::join(error0.span(), error1.span())
            },
            ParseError::UnicodeEscapeOutOfRange { span } => span.clone(),
            ParseError::InvalidUnicodeEscapeChar { span } => span.clone(),
            ParseError::UnclosedMultilineComment { span } => span.clone(),
            ParseError::ExpectedNewline { span } => span.clone(),
        }
    }
}


