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

pub fn big_uint(radix: u32) -> impl Parser<Output = BigUint> + Clone {
    let inner_digit = {
        digit(radix)
        .map(Some)
        .or(keyword("_").map(|()| None))
    };
    digit(radix)
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

fn digits() -> impl Parser<Output = (Option<BasePrefix>, BigUint, Span)> + Clone {
    base_prefix()
    .optional()
    .and_then(|base_prefix_opt: Option<BasePrefix>| {
        let radix = base_prefix_opt.as_ref().map(BasePrefix::radix).unwrap_or(10);

        big_uint(radix)
        .map_with_span(move |big_uint, span| {
            (base_prefix_opt.clone(), big_uint, span)
        })
    })
}

pub fn int_ty() -> impl Parser<Output = IntTy> + Clone {
    let i8_parser = {
        i8_token()
        .map(|i8_token| IntTy::I8(i8_token))
    };
    let i16_parser = {
        i16_token()
        .map(|i16_token| IntTy::I16(i16_token))
    };
    let i32_parser = {
        i32_token()
        .map(|i32_token| IntTy::I32(i32_token))
    };
    let i64_parser = {
        i64_token()
        .map(|i64_token| IntTy::I64(i64_token))
    };
    let u8_parser = {
        u8_token()
        .map(|u8_token| IntTy::U8(u8_token))
    };
    let u16_parser = {
        u16_token()
        .map(|u16_token| IntTy::U16(u16_token))
    };
    let u32_parser = {
        u32_token()
        .map(|u32_token| IntTy::U32(u32_token))
    };
    let u64_parser = {
        u64_token()
        .map(|u64_token| IntTy::U64(u64_token))
    };

    i8_parser
    .or(i16_parser)
    .or(i32_parser)
    .or(i64_parser)
    .or(u8_parser)
    .or(u16_parser)
    .or(u32_parser)
    .or(u64_parser)
}

pub fn int_literal() -> impl Parser<Output = IntLiteral> + Clone {
    numeric_sign()
    .optional()
    .then(digits())
    .then(int_ty().optional())
    .map(|((numeric_sign_opt, (base_prefix_opt, big_uint, digits_span)), ty_suffix_opt): ((Option<_>, _), Option<_>)| {
        let parsed = match numeric_sign_opt {
            Some(NumericSign::Negative { .. }) => -BigInt::from(big_uint),
            Some(NumericSign::Positive { .. }) | None => BigInt::from(big_uint),
        };
        IntLiteral { numeric_sign_opt, base_prefix_opt, digits_span, ty_suffix_opt, parsed }
    })
}
