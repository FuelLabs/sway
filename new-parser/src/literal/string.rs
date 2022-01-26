use crate::priv_prelude::*;

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
    .map(|((open_quote, parsed), close_quote)| {
        let WithSpan { parsed, span: contents_span } = parsed;
        StringLiteral { open_quote, contents_span, close_quote, parsed }
    })
}

fn string_literal_contents() -> impl Parser<Output = WithSpan<String>> + Clone {
    string_char()
    .repeated()
    .map(|chars_with_span: WithSpan<Vec<WithSpan<char>>>| {
        let WithSpan { parsed: chars, span } = chars_with_span;
        let s = {
            chars
            .into_iter()
            .map(|c_with_span| c_with_span.parsed)
            .collect()
        };
        WithSpan { parsed: s, span }
    })
}

fn string_char() -> impl Parser<Output = WithSpan<char>> + Clone {
    keyword("\\")
    .optional()
    .and_then(|backslash_res: Result<Span, Span>| match backslash_res {
        Ok(backslash_span) => {
            Either::Left(
                escape_code()
                .map(move |c_with_span| {
                    let WithSpan { parsed: c, span } = c_with_span;
                    WithSpan { parsed: c, span: Span::join(backslash_span.clone(), span) }
                })
            )
        },
        Err(..) => {
            Either::Right(single_char())
        },
    })
}

