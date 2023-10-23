use crate::{
    comments::rewrite_with_comments,
    formatter::*,
    utils::map::byte_span::{ByteSpan, LeafSpans},
};
use std::fmt::Write;
use sway_ast::{keywords::Token, TraitType};
use sway_types::Spanned;

impl Format for TraitType {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // Required for comment formatting
        let start_len = formatted_code.len();

        // Add the const token
        write!(formatted_code, "{} ", self.type_token.span().as_str())?;

        // Add name of the const
        self.name.format(formatted_code, formatter)?;

        // Check if ` = ` exists
        if let Some(eq_token) = &self.eq_token_opt {
            write!(formatted_code, " {} ", eq_token.ident().as_str())?;
        }

        // Check if ty exists
        if let Some(ty) = &self.ty_opt {
            ty.format(formatted_code, formatter)?;
        }

        write!(formatted_code, "{}", self.semicolon_token.ident().as_str())?;

        rewrite_with_comments::<TraitType>(
            formatter,
            self.span(),
            self.leaf_spans(),
            formatted_code,
            start_len,
        )?;
        Ok(())
    }
}

impl LeafSpans for TraitType {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        collected_spans.push(ByteSpan::from(self.type_token.span()));
        collected_spans.push(ByteSpan::from(self.name.span()));
        if let Some(eq_token) = &self.eq_token_opt {
            collected_spans.push(ByteSpan::from(eq_token.span()));
        }
        if let Some(ty) = &self.ty_opt {
            collected_spans.append(&mut ty.leaf_spans());
        }
        collected_spans
    }
}
