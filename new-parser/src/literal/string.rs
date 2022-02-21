use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct StringLiteral {
    pub open_quote: QuoteToken,
    pub contents_span: Span,
    pub close_quote: QuoteToken,
    pub parsed: String,
}

impl Spanned for StringLiteral {
    fn span(&self) -> Span {
        Span::join(self.open_quote.span(), self.close_quote.span())
    }
}

#[derive(Clone)]
pub struct ExpectedStringError {
    pub position: usize,
}

#[derive(Clone)]
pub enum StringError {
    InvalidEscapeCode {
        position: usize,
    },
    MalformedEscapeCode(EscapeCodeFatalError),
    UnclosedString {
        start_position: usize,
    },
}

pub fn string_literal() -> impl Parser<Output = StringLiteral> + Clone {
    quote_token()
    .map_err(|ExpectedQuoteTokenError { position }| ExpectedStringError { position })
    .then(string_literal_contents())
    .then(
        quote_token()
        .map_err(|_| unreachable!("string_literal_contents stops when it reaches a quote character"))
    )
    .map_fatal_err_with_span(|error, span| match error {
        StringCharError::InvalidEscapeCode { position } => StringError::InvalidEscapeCode { position },
        StringCharError::MalformedEscapeCode(error) => StringError::MalformedEscapeCode(error),
        StringCharError::UnclosedString => StringError::UnclosedString { start_position: span.start() },
    })
    .map(|((open_quote, (parsed, contents_span)), close_quote)| {
        StringLiteral { open_quote, contents_span, close_quote, parsed }
    })
}

fn string_literal_contents<E: Clone>()
    -> impl Parser<Output = (String, Span), Error = E, FatalError = StringCharError> + Clone
{
    string_char()
    .map_err(|UnexpectedCloseQuote| ())
    .repeated()
    .map_with_span(|chars: Vec<char>, span| {
        let s = {
            chars
            .into_iter()
            .collect()
        };
        (s, span)
    })
}

#[derive(Clone)]
struct UnexpectedCloseQuote;

#[derive(Clone)]
enum StringCharError {
    InvalidEscapeCode {
        position: usize,
    },
    MalformedEscapeCode(EscapeCodeFatalError),
    UnclosedString,
}

fn string_char() -> impl Parser<Output = char, Error = UnexpectedCloseQuote, FatalError = StringCharError> + Clone {
    let escape_code = {
        keyword("\\")
        .map_err(|ExpectedKeywordError { .. }| ())
        .then(
            escape_code()
            .map_err(|EscapeCodeError { position }| StringCharError::InvalidEscapeCode { position })
            .map_fatal_err(StringCharError::MalformedEscapeCode)
            .fatal()
        )
        .map(|((), c)| c)
    };
    let non_close_quote_char = {
        single_char()
        .map_err(|UnexpectedEofError| StringCharError::UnclosedString)
        .fatal()
        .try_map(|c| {
            if c == '"' {
                Err(Ok(()))
            } else {
                Ok(c)
            }
        })
    };
    or! {
        escape_code,
        non_close_quote_char,
    }
    .or_else(|((), ()), _span| Err(Ok(UnexpectedCloseQuote)))
}

