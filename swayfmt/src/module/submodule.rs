use crate::{
    formatter::*,
    utils::map::byte_span::{ByteSpan, LeafSpans},
};
use std::fmt::Write;
use sway_ast::{
    keywords::{Keyword, ModToken, SemicolonToken, Token},
    submodule::Submodule,
    PubToken,
};
use sway_types::Spanned;

impl Format for Submodule {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        if self.visibility.is_some() {
            write!(formatted_code, "{} ", PubToken::AS_STR)?;
        }
        write!(formatted_code, "{} ", ModToken::AS_STR)?;
        self.name.format(formatted_code, formatter)?;
        writeln!(formatted_code, "{}", SemicolonToken::AS_STR)?;
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
