use crate::{
    comments::rewrite_with_comments,
    formatter::*,
    utils::map::byte_span::{ByteSpan, LeafSpans},
};
use std::fmt::Write;
use sway_ast::{
    keywords::{ColonToken, ConstToken, EqToken, Keyword, SemicolonToken, Token},
    ItemConst, PubToken,
};
use sway_types::Spanned;

impl Format for ItemConst {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // Required for comment formatting
        let start_len = formatted_code.len();

        // Check if pub token exists, if so add it.
        if self.pub_token.is_some() {
            write!(formatted_code, "{} ", PubToken::AS_STR)?;
        }

        // Add the const token
        write!(formatted_code, "{} ", ConstToken::AS_STR)?;

        // Add name of the const
        self.name.format(formatted_code, formatter)?;

        // Check if ty exists
        if let Some((_colon_token, ty)) = &self.ty_opt {
            // Add colon
            write!(formatted_code, "{} ", ColonToken::AS_STR)?;
            ty.format(formatted_code, formatter)?;
        }

        // Check if ` = ` exists
        if self.eq_token_opt.is_some() {
            write!(formatted_code, " {} ", EqToken::AS_STR)?;
        }

        // Check if expression exists
        if let Some(expr) = &self.expr_opt {
            expr.format(formatted_code, formatter)?;
        }

        write!(formatted_code, "{}", SemicolonToken::AS_STR)?;

        rewrite_with_comments::<ItemConst>(
            formatter,
            self.span(),
            self.leaf_spans(),
            formatted_code,
            start_len,
        )?;
        Ok(())
    }
}

impl LeafSpans for ItemConst {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        if let Some(visibility) = &self.pub_token {
            collected_spans.push(ByteSpan::from(visibility.span()));
        }
        collected_spans.push(ByteSpan::from(self.const_token.span()));
        collected_spans.push(ByteSpan::from(self.name.span()));
        if let Some(ty) = &self.ty_opt {
            collected_spans.append(&mut ty.leaf_spans());
        }
        if let Some(eq_token) = &self.eq_token_opt {
            collected_spans.push(ByteSpan::from(eq_token.span()));
        }
        if let Some(expr) = &self.expr_opt {
            collected_spans.append(&mut expr.leaf_spans());
        }
        collected_spans.push(ByteSpan::from(self.semicolon_token.span()));
        collected_spans
    }
}
