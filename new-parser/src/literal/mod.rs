use crate::priv_prelude::*;

mod int;
mod string;

pub use int::*;
pub use string::*;

#[derive(Debug, Clone)]
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

pub struct ExpectedNumericSignError {
    pub position: usize,
}

pub fn numeric_sign<R>()
    -> impl Parser<Output = NumericSign, Error = ExpectedNumericSignError, FatalError = R> + Clone
{
    let positive = {
        add_token()
        .map(|add_token| NumericSign::Positive { add_token })
        .map_err(|ExpectedAddTokenError { .. }| ())
    };
    let negative = {
        sub_token()
        .map(|sub_token| NumericSign::Negative { sub_token })
        .map_err(|ExpectedSubTokenError { .. }| ())
    };

    or! {
        positive,
        negative,
    }
    .or_else(|(), span| Err(Ok(ExpectedNumericSignError { position: span.start() })))
}

pub struct ExpectedDigitError {
    pub position: usize,
}

pub fn digit<R>(radix: u32)
    -> impl Parser<Output = u32, Error = ExpectedDigitError, FatalError = R> + Clone
{
    single_char()
    .map_err_with_span(|UnexpectedEofError, span| ExpectedDigitError { position: span.start() })
    .try_map_with_span(move |c: char, span| match c.to_digit(radix) {
        Some(value) => Ok(value),
        None => Err(Ok(ExpectedDigitError { position: span.start() })),
    })
}

#[derive(Clone)]
pub struct EscapeCodeError {
    pub position: usize,
}

#[derive(Clone)]
pub enum EscapeCodeFatalError {
    ExpectedOpenBraceForUnicodeEscape  {
        position: usize,
    },
    ExpectedCloseBraceOrDigitInUnicodeEscape {
        position: usize,
    },
    ExpectedHexDigitForHexEscape {
        position: usize,
    },
    UnicodeEscapeOutOfRange {
        digits_span: Span,
    },
    InvalidUnicodeEscapeChar {
        digits_span: Span,
    },
}

pub fn escape_code()
    -> impl Parser<Output = char, Error = EscapeCodeError, FatalError = EscapeCodeFatalError> + Clone
{
    let newline = keyword("n").map(|()| '\n').map_err(|ExpectedKeywordError { .. }| ());
    let carriage_return = keyword("r").map(|()| '\r').map_err(|ExpectedKeywordError { .. }| ());
    let tab = keyword("t").map(|()| '\t').map_err(|ExpectedKeywordError { .. }| ());
    let backslash = keyword("\\").map(|()| '\\').map_err(|ExpectedKeywordError { .. }| ());
    let null = keyword("0").map(|()| '\0').map_err(|ExpectedKeywordError { .. }| ());
    let apostrophe = keyword("'").map(|()| '\'').map_err(|ExpectedKeywordError { .. }| ());
    let quote = keyword("\"").map(|()| '"').map_err(|ExpectedKeywordError { .. }| ());
    let hex = {
        keyword("x")
        .map_err(|ExpectedKeywordError { .. }| ())
        .then(
            digit(16)
            .then(digit(16))
            .map_err(|ExpectedDigitError { position }| {
                EscapeCodeFatalError::ExpectedHexDigitForHexEscape { position }
            })
            .fatal()
        )
        //.map(|(((), high), low)| char::try_from(high << 16 | low).unwrap())
        .map(|((), (high, low))| char::try_from(high << 16 | low).unwrap())
    };
    let unicode = {
        keyword("u")
        .map_err(|ExpectedKeywordError { .. }| ())
        .then(
            keyword("{")
            .map_err(|ExpectedKeywordError { position, .. }| {
                EscapeCodeFatalError::ExpectedOpenBraceForUnicodeEscape { position }
            })
            .then(
                digit(16)
                .map_err(|ExpectedDigitError { .. }| ())
                .repeated()
                .map_with_span(|digits, span| (digits, span))
            )
            .then(
                keyword("}")
                .map_err(|ExpectedKeywordError { position, .. }| {
                    EscapeCodeFatalError::ExpectedCloseBraceOrDigitInUnicodeEscape { position }
                })
            )
            .fatal()
        )
        .try_map(|((), (((), (digits, digits_span)), ()))| {
            let mut value = 0u32;
            for digit in digits {
                value = match value.checked_mul(16) {
                    Some(value) => value,
                    None => {
                        let error = EscapeCodeFatalError::UnicodeEscapeOutOfRange { digits_span };
                        return Err(Err(error));
                    },
                };
                value += digit;
            }
            match char::try_from(value) {
                Ok(c) => Ok(c),
                Err(_) => {
                    let error = EscapeCodeFatalError::InvalidUnicodeEscapeChar {
                        digits_span,
                    };
                    Err(Err(error))
                },
            }
        })
    };
    
    or! {
        newline,
        carriage_return,
        tab,
        backslash,
        null,
        apostrophe,
        quote,
        hex,
        unicode,
    }
    .or_else(|(), span| Err(Ok(EscapeCodeError { position: span.start() })))
}

#[derive(Clone, Debug)]
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

pub struct ExpectedBasePrefixError {
    pub position: usize,
}

pub fn base_prefix<R>()
    -> impl Parser<Output = BasePrefix, Error = ExpectedBasePrefixError, FatalError = R> + Clone
{
    let hex = {
        hex_prefix_token()
        .map(BasePrefix::Hex)
        .map_err(|ExpectedHexPrefixTokenError { .. }| ())
    };
    let octal = {
        octal_prefix_token()
        .map(BasePrefix::Octal)
        .map_err(|ExpectedOctalPrefixTokenError { .. }| ())
    };
    let binary = {
        binary_prefix_token()
        .map(BasePrefix::Binary)
        .map_err(|ExpectedBinaryPrefixTokenError { .. }| ())
    };
    
    or! {
        hex,
        octal,
        binary,
    }
    .or_else(|(), span| Err(Ok(ExpectedBasePrefixError { position: span.start() })))
}

