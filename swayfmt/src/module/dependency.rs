use crate::{
    formatter::*,
    utils::map::byte_span::{ByteSpan, LeafSpans},
};
use std::fmt::Write;
use sway_ast::{dependency::DependencyPath, Dependency};
use sway_types::Spanned;

impl Format for Dependency {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(formatted_code, "{} ", self.dep_token.span().as_str())?;
        self.path.format(formatted_code, formatter)?;
        writeln!(formatted_code, "{}", self.semicolon_token.span().as_str())?;

        Ok(())
    }
}

impl Format for DependencyPath {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        self.prefix.format(formatted_code, formatter)?;
        for (forward_slash_token, ident) in &self.suffixes {
            write!(formatted_code, "{}", forward_slash_token.span().as_str())?;
            ident.format(formatted_code, formatter)?;
        }

        Ok(())
    }
}

impl LeafSpans for Dependency {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.dep_token.span())];
        collected_spans.append(&mut self.path.leaf_spans());
        collected_spans.push(ByteSpan::from(self.semicolon_token.span()));
        collected_spans
    }
}

impl LeafSpans for DependencyPath {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = self.prefix.leaf_spans();
        collected_spans.append(&mut self.suffixes.leaf_spans());
        collected_spans
    }
}
