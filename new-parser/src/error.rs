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
    InvalidEscapeCode {
        span: Span,
    },
    UnclosedMultilineComment {
        span: Span,
    },
    UnknownOpcode {
        span: Span,
    },
    ExpectedExpression {
        span: Span,
    },
    ExpectedPattern {
        span: Span,
    },
    ExpectedItem {
        span: Span,
    },
    MalformedImport {
        span: Span,
    },
    ExpectedStatement {
        span: Span,
    },
    UnexpectedQuote {
        span: Span,
    },
    ExpectedType {
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
            ParseError::InvalidEscapeCode { span } => span.clone(),
            ParseError::UnclosedMultilineComment { span } => span.clone(),
            ParseError::UnknownOpcode { span } => span.clone(),
            ParseError::ExpectedExpression { span } => span.clone(),
            ParseError::ExpectedPattern { span } => span.clone(),
            ParseError::ExpectedItem { span } => span.clone(),
            ParseError::MalformedImport { span } => span.clone(),
            ParseError::ExpectedStatement { span } => span.clone(),
            ParseError::UnexpectedQuote { span } => span.clone(),
            ParseError::ExpectedType { span } => span.clone(),
        }
    }
}

impl ParseError {
    fn build_report(&self, builder: ReportBuilder<Span>) -> ReportBuilder<Span> {
        let builder = match self {
            ParseError::ExpectedKeyword { word, .. } => {
                builder
                .with_message(format!("expected {:?}", word))
            },
            ParseError::UnexpectedEof { .. } => {
                builder
                .with_message("unexpected end of file")
            },
            ParseError::ExpectedEof { .. } => {
                builder
                .with_message("expected end of file")
            },
            ParseError::ExpectedIdent { .. } => {
                builder
                .with_message("expected an identifier")
            },
            ParseError::ExpectedWhitespace { .. } => {
                builder
                .with_message("expected whitespace")
            },
            ParseError::ExpectedDigit { .. } => {
                builder
                .with_message("expected a digit")
            },
            ParseError::Or { error0, error1 } => {
                error1.build_report(error0.build_report(builder))
            },
            ParseError::UnicodeEscapeOutOfRange { .. } => {
                builder
                .with_message("unicode escape out of range")
            },
            ParseError::InvalidUnicodeEscapeChar { .. } => {
                builder
                .with_message("invalid unicode escape character")
            },
            ParseError::InvalidEscapeCode { .. } => {
                builder
                .with_message("invalid escape code")
            },
            ParseError::UnclosedMultilineComment { .. } => {
                builder
                .with_message("unclosed multi-line comment")
            },
            ParseError::UnknownOpcode { .. } => {
                builder
                .with_message("unknown op code")
            },
            ParseError::ExpectedExpression { .. } => {
                builder
                .with_message("expected an expression")
            },
            ParseError::ExpectedPattern { .. } => {
                builder
                .with_message("expected a pattern")
            },
            ParseError::ExpectedItem { .. } => {
                builder
                .with_message("expected an item")
            },
            ParseError::MalformedImport { .. } => {
                builder
                .with_message("malformed input")
            },
            ParseError::ExpectedStatement { .. } => {
                builder
                .with_message("expected a statement")
            },
            ParseError::UnexpectedQuote { .. } => {
                builder
                .with_message("unexpected quote")
            },
            ParseError::ExpectedType { .. } => {
                builder
                .with_message("expected a type")
            },
        };
        builder.with_label(ariadne::Label::new(self.span()))
    }

    pub fn report(&self) -> Report<Span> {
        let span = self.span();
        let builder = Report::build(ReportKind::Error, span.src().clone(), ariadne::Span::start(&span));
        let builder = builder.with_config(ariadne::Config::default());
        let builder = self.build_report(builder);
        builder.finish()
    }
}

