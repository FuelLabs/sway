use crate::{
    fmt::*,
    utils::comments::{ByteSpan, LeafSpans},
};
use std::fmt::Write;
use sway_ast::Literal;

impl Format for Literal {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            // TODO: do more digging into `Literal` and see if there is more formatting to do.
            Self::String(lit_string) => write!(formatted_code, "{}", lit_string.span.as_str())?,
            Self::Char(lit_char) => write!(formatted_code, "{}", lit_char.span.as_str())?,
            Self::Int(lit_int) => write!(formatted_code, "{}", lit_int.span.as_str())?,
            Self::Bool(lit_bool) => write!(formatted_code, "{}", lit_bool.span.as_str())?,
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
