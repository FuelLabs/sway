use crate::fmt::*;
use std::fmt::Write;
use sway_parse::Literal;

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
