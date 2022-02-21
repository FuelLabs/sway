use crate::priv_prelude::*;

#[derive(Debug, Clone)]
pub struct IntLiteral {
    pub numeric_sign_opt: Option<NumericSign>,
    pub base_prefix_opt: Option<BasePrefix>,
    pub digits_span: Span,
    pub ty_suffix_opt: Option<IntTy>,
    pub parsed: BigInt,
}

#[derive(Debug, Clone)]
pub enum IntTy {
    I8(I8Token),
    I16(I16Token),
    I32(I32Token),
    I64(I64Token),
    U8(U8Token),
    U16(U16Token),
    U32(U32Token),
    U64(U64Token),
}

impl Spanned for IntLiteral {
    fn span(&self) -> Span {
        let first = {
            self.numeric_sign_opt.as_ref().map(Spanned::span)
            .or_else(|| self.base_prefix_opt.as_ref().map(Spanned::span))
            .unwrap_or_else(|| self.digits_span.clone())
        };
        let last = {
            self.ty_suffix_opt.as_ref().map(Spanned::span)
            .unwrap_or_else(|| self.digits_span.clone())
        };
        Span::join(first, last)
    }
}

impl Spanned for IntTy {
    fn span(&self) -> Span {
        match self {
            IntTy::I8(i8_token) => i8_token.span(),
            IntTy::I16(i16_token) => i16_token.span(),
            IntTy::I32(i32_token) => i32_token.span(),
            IntTy::I64(i64_token) => i64_token.span(),
            IntTy::U8(u8_token) => u8_token.span(),
            IntTy::U16(u16_token) => u16_token.span(),
            IntTy::U32(u32_token) => u32_token.span(),
            IntTy::U64(u64_token) => u64_token.span(),
        }
    }
}

#[derive(Clone)]
pub struct ExpectedIntLiteralError {
    pub position: usize,
}

#[derive(Clone)]
pub struct ExpectedIntLiteralDigitsError {
    pub position: usize,
}

pub fn int_literal()
    -> impl Parser<
        Output = IntLiteral,
        Error = ExpectedIntLiteralError,
        FatalError = ExpectedIntLiteralDigitsError,
    > + Clone {
    numeric_sign()
    .map_err(|ExpectedNumericSignError { .. }| ())
    .optional()
    .then(digits())
    .map_err(|ExpectedDigitsError { position }| ExpectedIntLiteralError { position })
    .map_fatal_err(|ExpectedBigUintError { position }| ExpectedIntLiteralDigitsError { position })
    .then(int_ty().optional())
    .map(|((numeric_sign_opt, (base_prefix_opt, big_uint, digits_span)), ty_suffix_opt): ((Option<_>, _), Option<_>)| {
        let parsed = match numeric_sign_opt {
            Some(NumericSign::Negative { .. }) => -BigInt::from(big_uint),
            Some(NumericSign::Positive { .. }) | None => BigInt::from(big_uint),
        };
        IntLiteral { numeric_sign_opt, base_prefix_opt, digits_span, ty_suffix_opt, parsed }
    })
}

pub fn int_ty<R>() -> impl Parser<Output = IntTy, Error = (), FatalError = R> + Clone {
    let i8_parser = {
        i8_token()
        .map(|i8_token| IntTy::I8(i8_token))
        .map_err(|ExpectedI8TokenError { .. }| ())
    };
    let i16_parser = {
        i16_token()
        .map(|i16_token| IntTy::I16(i16_token))
        .map_err(|ExpectedI16TokenError { .. }| ())
    };
    let i32_parser = {
        i32_token()
        .map(|i32_token| IntTy::I32(i32_token))
        .map_err(|ExpectedI32TokenError { .. }| ())
    };
    let i64_parser = {
        i64_token()
        .map(|i64_token| IntTy::I64(i64_token))
        .map_err(|ExpectedI64TokenError { .. }| ())
    };
    let u8_parser = {
        u8_token()
        .map(|u8_token| IntTy::U8(u8_token))
        .map_err(|ExpectedU8TokenError { .. }| ())
    };
    let u16_parser = {
        u16_token()
        .map(|u16_token| IntTy::U16(u16_token))
        .map_err(|ExpectedU16TokenError { .. }| ())
    };
    let u32_parser = {
        u32_token()
        .map(|u32_token| IntTy::U32(u32_token))
        .map_err(|ExpectedU32TokenError { .. }| ())
    };
    let u64_parser = {
        u64_token()
        .map(|u64_token| IntTy::U64(u64_token))
        .map_err(|ExpectedU64TokenError { .. }| ())
    };

    or! {
        i8_parser,
        i16_parser,
        i32_parser,
        i64_parser,
        u8_parser,
        u16_parser,
        u32_parser,
        u64_parser,
    }
    .map_err(|((), (), (), (), (), (), (), ())| ())
}

#[derive(Clone)]
struct ExpectedDigitsError {
    position: usize,
}

fn digits()
    -> impl Parser<Output = (Option<BasePrefix>, BigUint, Span), Error = ExpectedDigitsError, FatalError = ExpectedBigUintError> + Clone
{
    base_prefix()
    .map_err(|ExpectedBasePrefixError { .. }| ())
    .optional()
    .and_then(|base_prefix_opt: Option<BasePrefix>| {
        match &base_prefix_opt {
            Some(base_prefix) => {
                Either::Left(big_uint(base_prefix.radix()).fatal())
            },
            None => {
                Either::Right(big_uint(10).map_err(|ExpectedBigUintError { position }| ExpectedDigitsError { position }))
            },
        }
        .map_with_span(move |big_uint, span| (base_prefix_opt.clone(), big_uint, span))
    })
}

#[derive(Clone)]
pub struct ExpectedBigUintError {
    pub position: usize,
}

pub fn big_uint<R>(radix: u32)
    -> impl Parser<Output = BigUint, Error = ExpectedBigUintError, FatalError = R> + Clone
{
    let inner_digit = {
        or! {
            digit(radix).map(Some).map_err(|ExpectedDigitError { .. }| ()),
            keyword("_").map(|()| None).map_err(|ExpectedKeywordError { .. }| ()),
        }
        .map_err(|((), ())| ())
    };
    digit(radix)
    .map_err(|ExpectedDigitError { position }| ExpectedBigUintError { position })
    .then(inner_digit.repeated())
    .map(move |(first_digit, digits)| {
        let mut value = BigUint::from(first_digit);
        for digit_opt in digits {
            if let Some(digit) = digit_opt {
                value *= radix;
                value += digit;
            }
        }
        value
    })
}

