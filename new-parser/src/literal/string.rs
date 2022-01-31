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

pub fn string_literal() -> impl Parser<Output = StringLiteral> + Clone {
    quote_token()
    .then(string_literal_contents())
    .then(quote_token())
    .map(|((open_quote, (parsed, contents_span)), close_quote)| {
        StringLiteral { open_quote, contents_span, close_quote, parsed }
    })
}

fn string_literal_contents() -> impl Parser<Output = (String, Span)> + Clone {
    string_char()
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

fn string_char() -> impl Parser<Output = char> + Clone {
    keyword("\\")
    .optional()
    .and_then(|backslash_opt: Option<()>| match backslash_opt {
        Some(()) => Either::Left(escape_code()),
        None => Either::Right(single_char()),
    })
}

