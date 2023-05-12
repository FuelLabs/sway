use crate::{
    formatter::*,
    utils::map::byte_span::{ByteSpan, LeafSpans},
};
use std::fmt::Write;
use sway_ast::submodule::Submodule;
use sway_types::Spanned;

impl Format for Submodule {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(formatted_code, "{} ", self.mod_token.span().as_str())?;
        self.name.format(formatted_code, formatter)?;
        writeln!(formatted_code, "{}", self.semicolon_token.span().as_str())?;
        Ok(())
    }
}

impl LeafSpans for Submodule {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        vec![
            ByteSpan::from(self.mod_token.span()),
            ByteSpan::from(self.name.span()),
            ByteSpan::from(self.semicolon_token.span()),
        ]
    }
}
