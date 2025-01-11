use crate::{
    comments::rewrite_with_comments,
    formatter::*,
    utils::map::byte_span::{ByteSpan, LeafSpans},
};
use std::fmt::Write;
use sway_ast::{
    keywords::{EqToken, Keyword, SemicolonToken, Token, TypeToken},
    TraitType,
};
use sway_types::Spanned;

impl Format for TraitType {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // Required for comment formatting
        let start_len = formatted_code.len();

        // Add the type token
        write!(formatted_code, "{} ", TypeToken::AS_STR)?;

        // Add name of the type
        self.name.format(formatted_code, formatter)?;

        // Check if ` = ` exists
        if self.eq_token_opt.is_some() {
            write!(formatted_code, " {} ", EqToken::AS_STR)?;
        }

        // Check if ty exists
        if let Some(ty) = &self.ty_opt {
            ty.format(formatted_code, formatter)?;
        }

        write!(formatted_code, "{}", SemicolonToken::AS_STR)?;

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
