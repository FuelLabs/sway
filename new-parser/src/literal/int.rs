use crate::priv_prelude::*;

pub struct IntLiteral {
    pub numeric_sign_opt: Option<NumericSign>,
    pub base_prefix_opt: Option<BasePrefix>,
    pub digits_span: Span,
    pub ty_suffix_opt: Option<IntTy>,
    pub parsed: BigInt,
}

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

fn digits() -> impl Parser<char, (Option<BasePrefix>, BigUint, Span), Error = Cheap<char, Span>> + Clone {
    let hex = {
        hex_prefix_token()
        .then(big_uint(16).map_with_span(|big_uint, span| (big_uint, span)))
        .map(|(hex_prefix_token, (big_uint, span))| {
            (Some(BasePrefix::Hex(hex_prefix_token)), big_uint, span)
        })
    };
    let octal = {
        octal_prefix_token()
        .then(big_uint(8).map_with_span(|big_uint, span| (big_uint, span)))
        .map(|(octal_prefix_token, (big_uint, span))| {
            (Some(BasePrefix::Octal(octal_prefix_token)), big_uint, span)
        })
    };
    let binary = {
        binary_prefix_token()
        .then(big_uint(2).map_with_span(|big_uint, span| (big_uint, span)))
        .map(|(binary_prefix_token, (big_uint, span))| {
            (Some(BasePrefix::Binary(binary_prefix_token)), big_uint, span)
        })
    };
    let decimal = {
        big_uint(10)
        .map_with_span(|big_uint, span| (None, big_uint, span))
    };
    
    hex
    .or(octal)
    .or(binary)
    .or(decimal)
}

pub fn int_literal() -> impl Parser<char, IntLiteral, Error = Cheap<char, Span>> + Clone {
    numeric_sign()
    .or_not()
    .then(digits())
    .then(int_ty().or_not())
    .map(|((numeric_sign_opt, (base_prefix_opt, big_uint, digits_span)), ty_suffix_opt)| {
        let parsed = match numeric_sign_opt {
            Some(NumericSign::Negative { .. }) => -BigInt::from(big_uint),
            Some(NumericSign::Positive { .. }) | None => BigInt::from(big_uint),
        };
        IntLiteral { numeric_sign_opt, base_prefix_opt, digits_span, ty_suffix_opt, parsed }
    })
}

pub fn big_uint(radix: u32) -> impl Parser<char, BigUint, Error = Cheap<char, Span>> + Clone {
    digit(radix)
    .repeated()
    .map(move |digits| {
        let mut value = BigUint::zero();
        for digit in digits {
            value *= 1u32 << radix;
            value += digit;
        }
        value
    })
}

pub fn base_prefix() -> impl Parser<char, BasePrefix, Error = Cheap<char, Span>> + Clone {
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

/*
pub fn int_digits() -> impl Parser<char, IntDigits, Error = Cheap<char, Span>> {
    let decimal = {
        chumsky::text::int(10)
        .map_with_span(|_, digits| IntDigits::Decimal { digits })
    };
    let hex = {
        hex_prefix_token()
        .then(chumsky::text::int(16).map_with_span(|_, digits| digits))
        .map(|(hex_prefix_token, digits)| IntDigits::Hex { hex_prefix_token, digits })
    };
    let octal = {
        octal_prefix_token()
        .then(chumsky::text::int(8).map_with_span(|_, digits| digits))
        .map(|(octal_prefix_token, digits)| IntDigits::Octal { octal_prefix_token, digits })
    };
    let binary = {
        binary_prefix_token()
        .then(chumsky::text::int(2).map_with_span(|_, digits| digits))
        .map(|(binary_prefix_token, digits)| IntDigits::Binary { binary_prefix_token, digits })
    };

    binary
    .or(octal)
    .or(hex)
    .or(decimal)
}
*/

pub fn int_ty() -> impl Parser<char, IntTy, Error = Cheap<char, Span>> + Clone {
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

