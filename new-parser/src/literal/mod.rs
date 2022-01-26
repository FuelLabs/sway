use crate::priv_prelude::*;

mod int;
mod string;

pub use int::*;
pub use string::*;

pub enum NumericSign {
    Positive {
        add_token: AddToken,
    },
    Negative {
        sub_token: SubToken,
    },
}

impl Spanned for NumericSign {
    fn span(&self) -> Span {
        match self {
            NumericSign::Positive { add_token } => add_token.span(),
            NumericSign::Negative { sub_token } => sub_token.span(),
        }
    }
}

pub fn numeric_sign() -> impl Parser<Output = NumericSign> + Clone {
    let positive = {
        add_token()
        .map(|add_token| NumericSign::Positive { add_token })
    };
    let negative = {
        sub_token()
        .map(|sub_token| NumericSign::Negative { sub_token })
    };

    positive
    .or(negative)
}

pub fn digit(radix: u32) -> impl Parser<Output = WithSpan<u32>> + Clone {
    single_char()
    .try_map(move |char_with_span: WithSpan<char>| {
        char_with_span.try_map(|c, span| {
            match c.to_digit(radix) {
                Some(value) => Ok(value),
                None => Err(ParseError::ExpectedDigit { span }),
            }
        })
    })
}

pub fn escape_code() -> impl Parser<Output = WithSpan<char>> + Clone {
    let newline = keyword("n").map(|span| WithSpan { parsed: '\n', span });
    let carriage_return = keyword("r").map(|span| WithSpan { parsed: '\r', span });
    let tab = keyword("t").map(|span| WithSpan { parsed: '\t', span });
    let backslash = keyword("\\").map(|span| WithSpan { parsed: '\\', span });
    let null = keyword("0").map(|span| WithSpan { parsed: '\0', span });
    let apostrophe = keyword("'").map(|span| WithSpan { parsed: '\'', span });
    let quote = keyword("\"").map(|span| WithSpan { parsed: '\"', span });
    let hex = {
        keyword("x")
        .then(digit(16))
        .then(digit(16))
        .map(|((span_start, high_with_span), low_with_span)| {
            let WithSpan { parsed: high, span: _ } = high_with_span;
            let WithSpan { parsed: low, span: low_span } = low_with_span;
            let c = char::try_from(high << 16 | low).unwrap();
            WithSpan { parsed: c, span: Span::join(span_start, low_span) }
        })
    };
    let unicode = {
        keyword("u")
        .then(keyword("{"))
        .then(digit(16).repeated())
        .then(keyword("}"))
        .try_map(|(((span_start, _), digits), span_end)| {
            let WithSpan { parsed: digits, span: digits_span } = digits;
            let mut value = 0u32;
            for digit in digits {
                value = match value.checked_mul(16) {
                    Some(value) => value,
                    None => return Err(ParseError::UnicodeEscapeOutOfRange {
                        span: digits_span,
                    }),
                };
                let WithSpan { parsed: digit, span: _ } = digit;
                value += digit;
            }
            match char::try_from(value) {
                Ok(c) => {
                    let span = Span::join(span_start, span_end);
                    Ok(WithSpan { parsed: c, span })
                },
                Err(_) => Err(ParseError::InvalidUnicodeEscapeChar {
                    span: digits_span,
                }),
            }
        })
    };
    
    newline
    .or(carriage_return)
    .or(tab)
    .or(backslash)
    .or(null)
    .or(apostrophe)
    .or(quote)
    .or(hex)
    .or(unicode)
}

#[derive(Clone)]
pub enum BasePrefix {
    Hex(HexPrefixToken),
    Octal(OctalPrefixToken),
    Binary(BinaryPrefixToken),
}

impl Spanned for BasePrefix {
    fn span(&self) -> Span {
        match self {
            BasePrefix::Hex(hex_prefix_token) => hex_prefix_token.span(),
            BasePrefix::Octal(octal_prefix_token) => octal_prefix_token.span(),
            BasePrefix::Binary(binary_prefix_token) => binary_prefix_token.span(),
        }
    }
}

impl BasePrefix {
    pub fn radix(&self) -> u32 {
        match self {
            BasePrefix::Hex(..) => 16,
            BasePrefix::Octal(..) => 8,
            BasePrefix::Binary(..) => 2,
        }
    }
}

pub fn base_prefix() -> impl Parser<Output = BasePrefix> + Clone {
    let hex = {
        hex_prefix_token()
        .map(BasePrefix::Hex)
    };
    let octal = {
        octal_prefix_token()
        .map(BasePrefix::Octal)
    };
    let binary = {
        binary_prefix_token()
        .map(BasePrefix::Binary)
    };
    
    hex
    .or(octal)
    .or(binary)
}

