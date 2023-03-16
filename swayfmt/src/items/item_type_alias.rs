use crate::{
    comments::rewrite_with_comments,
    formatter::*,
    utils::map::byte_span::{ByteSpan, LeafSpans},
};
use std::fmt::Write;
use sway_ast::{keywords::Token, ItemTypeAlias};
use sway_types::Spanned;

impl Format for ItemTypeAlias {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // Required for comment formatting
        let start_len = formatted_code.len();

        // Check if visibility token exists if so add it.
        if let Some(visibility_token) = &self.visibility {
            write!(formatted_code, "{} ", visibility_token.span().as_str())?;
        }

        // Add the `type` token
        write!(formatted_code, "{} ", self.type_token.span().as_str())?;

        // Add name of the type alias
        self.name.format(formatted_code, formatter)?;

        // Add the `=` token
        write!(formatted_code, " {} ", self.eq_token.ident().as_str())?;

        // Format and add `ty`
        self.ty.format(formatted_code, formatter)?;

        // Add the `;` token
        write!(formatted_code, "{}", self.semicolon_token.ident().as_str())?;

        rewrite_with_comments::<ItemTypeAlias>(
            formatter,
            self.span(),
            self.leaf_spans(),
            formatted_code,
            start_len,
        )?;
        Ok(())
    }
}

impl LeafSpans for ItemTypeAlias {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        if let Some(visibility) = &self.visibility {
            collected_spans.push(ByteSpan::from(visibility.span()));
        }
        collected_spans.push(ByteSpan::from(self.type_token.span()));
        collected_spans.push(ByteSpan::from(self.name.span()));
        collected_spans.push(ByteSpan::from(self.eq_token.span()));
        collected_spans.append(&mut self.ty.leaf_spans());
        collected_spans.push(ByteSpan::from(self.semicolon_token.span()));
        collected_spans
    }
}
