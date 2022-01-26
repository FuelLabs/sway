use crate::priv_prelude::*;

pub struct IntLiteral {
    pub numeric_sign_opt: Option<NumericSign>,
    pub base_prefix_opt: Option<BasePrefix>,
    pub digits_span: Span,
    pub ty_suffix_opt: Option<IntTy>,
    pub parsed: BigInt,
}

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

fn digits() -> impl Parser<Output = WithSpan<(Option<BasePrefix>, WithSpan<BigUint>)>> + Clone {
    base_prefix()
    .optional()
    .and_then(|base_prefix_res: Result<BasePrefix, Span>| {
        let span_start = base_prefix_res.span();
        let base_prefix_opt = base_prefix_res.ok();
        let radix = base_prefix_opt.as_ref().map(BasePrefix::radix).unwrap_or(10);

        big_uint(radix)
        .map(move |big_uint_with_span: WithSpan<BigUint>| {
            let span_end = big_uint_with_span.span();
            WithSpan {
                parsed: (base_prefix_opt.clone(), big_uint_with_span),
                span: Span::join(span_start.clone(), span_end),
            }
        })
    })
}

pub fn int_literal() -> impl Parser<Output = IntLiteral> + Clone {
    numeric_sign()
    .optional()
    .then(digits())
    .then(int_ty().optional())
    .map(|((numeric_sign_res, digits_with_span), ty_suffix_res): ((Result<_, _>, _), Result<_, _>)| {
        let numeric_sign_opt = numeric_sign_res.ok();
        let WithSpan { parsed: (base_prefix_opt, big_uint_with_span), span: _ } = digits_with_span;
        let WithSpan { parsed: big_uint, span: digits_span } = big_uint_with_span;
        let ty_suffix_opt = ty_suffix_res.ok();
        let parsed = match numeric_sign_opt {
            Some(NumericSign::Negative { .. }) => -BigInt::from(big_uint),
            Some(NumericSign::Positive { .. }) | None => BigInt::from(big_uint),
        };
        IntLiteral { numeric_sign_opt, base_prefix_opt, digits_span, ty_suffix_opt, parsed }
    })
}

pub fn big_uint(radix: u32) -> impl Parser<Output = WithSpan<BigUint>> + Clone {
    digit(radix)
    .repeated()
    .map(move |digits_with_span| {
        let WithSpan { parsed: digits, span } = digits_with_span;
        let mut value = BigUint::zero();
        for digit_with_span in digits {
            let WithSpan { parsed: digit, span: _ } = digit_with_span;
            value *= radix;
            value += digit;
        }
        WithSpan { parsed: value, span }
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

