use crate::priv_prelude::*;

/*
mod int;
mod string;
*/

/*
pub use int::*;
pub use string::*;
*/

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
    let code = {
        let newline = keyword("n").map(|()| '\n');
        let carriage_return = keyword("r").map(|()| '\r');
        let tab = keyword("t").map(|()| '\t');
        let backslash = keyword("\\").map(|()| '\\');
        let null = keyword("0").map(|()| '\0');
        let apostrophe = keyword("'").map(|()| '\'');
        let quote = keyword("\"").map(|()| '"');
        let hex = {
            keyword("x")
            .then(
                digit(16)
                .then(digit(16))
                .map(|(high_with_span, low_with_span)| {
                    let WithSpan { parsed: high, span: high_span } = high_with_span;
                    let WithSpan { parsed: low, span: low_span } = low_with_span;
                    let c = char::try_from(high << 16 | low).unwrap();
                    WithSpan { parsed: c, span: Span::join(high_span, low_span) }
                })
            )
            .map(|((), c)| c)
        };
        let unicode = {
            keyword("u")
            .then(keyword("{"))
            .then(digit(16).repeated().map_with_span(|digits, span| (digits, span)))
            .then(keyword("}"))
            .try_map(|((((), ()), (digits, digits_span)), ()), _span| {
                let mut value = 0u32;
                for digit in digits {
                    value = match value.checked_mul(16) {
                        Some(value) => value,
                        None => return Err(Cheap::expected_input_found(digits_span, [], None)),
                    };
                    value = match value.checked_add(digit) {
                        Some(value) => value,
                        None => return Err(Cheap::expected_input_found(digits_span, [], None)),
                    };
                }
                match char::try_from(value) {
                    Ok(c) => Ok(c),
                    Err(_) => Err(Cheap::expected_input_found(digits_span, [], None)),
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
    };

    keyword("\\")
    .then(code)
    .map(|((), c)| c)
}
