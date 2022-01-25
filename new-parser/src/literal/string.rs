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

pub fn string_literal() -> impl Parser<char, StringLiteral, Error = Cheap<char, Span>> + Clone {
    quote_token()
    .then(string_literal_contents().map_with_span(|parsed, span| (parsed, span)))
    .then(quote_token())
    .map(|((open_quote, (parsed, contents_span)), close_quote)| {
        StringLiteral { open_quote, contents_span, close_quote, parsed }
    })
}

fn string_literal_contents() -> impl Parser<char, String, Error = Cheap<char, Span>> + Clone {
    string_char().repeated().map(|chars| chars.into_iter().collect())
}

fn string_char() -> impl Parser<char, char, Error = Cheap<char, Span>> + Clone {
    escape_code()
    .or(chumsky::primitive::none_of("\""))
}

