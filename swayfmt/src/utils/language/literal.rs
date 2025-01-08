use crate::{
    formatter::*,
    utils::map::byte_span::{ByteSpan, LeafSpans},
};
use std::fmt::Write;
use sway_ast::{literal::LitBoolType, LitIntType, Literal};

impl Format for Literal {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Self::String(lit_string) => write!(formatted_code, "\"{}\"", lit_string.parsed)?,
            Self::Char(lit_char) => write!(formatted_code, "\'{}\'", lit_char.parsed)?,
            Self::Int(lit_int) => {
                // It is tricky to support formatting of `LitInt` for an arbitrary `LitInt`
                // that is potentially not backed by source code, but constructed in-memory.
                //
                // E.g., a constructed `LitInt` representing 1000 can have only the numeric value
                // (LitInt::parsed) specified, in which case we can simply output the value.
                // If it has the type specified (LitInt::ty_opt), we can output the type next to the
                // value, e.g., 1000u16.
                // But a `LitInt` backed by code can have an arbitrary representation that includes
                // underscores. E.g., 1_00_00__u16. In that case we need to preserve the original
                // representation.
                //
                // The taken approach is the following. If the length of the `LitInt::span` is zero,
                // we assume that it is not backed by source code and render the canonical representation,
                // 1000u16 in the above example. Otherwise, we assume that it is backed by source code
                // and use the actual spans to obtain the strings.

                if lit_int.span.is_empty() {
                    write!(formatted_code, "{}", lit_int.parsed)?;
                    if let Some((int_type, _)) = &lit_int.ty_opt {
                        let int_type = match int_type {
                            LitIntType::U8 => "u8",
                            LitIntType::U16 => "u16",
                            LitIntType::U32 => "u32",
                            LitIntType::U64 => "u64",
                            LitIntType::U256 => "u256",
                            LitIntType::I8 => "i8",
                            LitIntType::I16 => "i16",
                            LitIntType::I32 => "i32",
                            LitIntType::I64 => "i64",
                        };
                        write!(formatted_code, "{}", int_type)?;
                    }
                } else {
                    write!(formatted_code, "{}", lit_int.span.as_str())?;
                    if let Some((_, ty_span)) = &lit_int.ty_opt {
                        write!(formatted_code, "{}", ty_span.as_str())?;
                    }
                }
            }
            Self::Bool(lit_bool) => write!(
                formatted_code,
                "{}",
                match lit_bool.kind {
                    LitBoolType::True => "true",
                    LitBoolType::False => "false",
                }
            )?,
        }
        Ok(())
    }
}

impl LeafSpans for Literal {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        match self {
            Literal::String(str_lit) => vec![ByteSpan::from(str_lit.span.clone())],
            Literal::Char(chr_lit) => vec![ByteSpan::from(chr_lit.span.clone())],
            Literal::Int(int_lit) => vec![ByteSpan::from(int_lit.span.clone())],
            Literal::Bool(bool_lit) => vec![ByteSpan::from(bool_lit.span.clone())],
        }
    }
}
