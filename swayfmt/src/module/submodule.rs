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
        if let Some(pub_token) = &self.visibility {
            write!(formatted_code, "{} ", pub_token.span().as_str())?;
        }
        write!(formatted_code, "{} ", self.mod_token.span().as_str())?;
        self.name.format(formatted_code, formatter)?;
        writeln!(formatted_code, "{}", self.semicolon_token.span().as_str())?;
        Ok(())
    }
}

impl LeafSpans for Submodule {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut spans = Vec::with_capacity(4);
        if let Some(visibility) = &self.visibility {
            spans.push(ByteSpan::from(visibility.span()));
        }
        spans.extend_from_slice(&[
            ByteSpan::from(self.mod_token.span()),
            ByteSpan::from(self.name.span()),
            ByteSpan::from(self.semicolon_token.span()),
        ]);
        spans
    }
}
