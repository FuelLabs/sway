use crate::{
    comments::rewrite_with_comments,
    formatter::*,
    utils::map::byte_span::{ByteSpan, LeafSpans},
};
use std::fmt::Write;
use sway_ast::{keywords::Token, ItemConst};
use sway_types::Spanned;

impl Format for ItemConst {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let last = formatted_code.len();

        // Check if visibility token exists if so add it.
        if let Some(visibility_token) = &self.visibility {
            write!(formatted_code, "{} ", visibility_token.span().as_str())?;
        }

        // Add the const token
        write!(formatted_code, "{} ", self.const_token.span().as_str())?;

        // Add name of the const
        self.name.format(formatted_code, formatter)?;

        // Check if ty exists
        if let Some((colon_token, ty)) = &self.ty_opt {
            // Add colon
            write!(formatted_code, "{} ", colon_token.ident().as_str())?;
            ty.format(formatted_code, formatter)?;
        }

        // Check if ` = ` exists
        if let Some(eq_token) = &self.eq_token_opt {
            write!(formatted_code, " {} ", eq_token.ident().as_str())?;
        }

        // Check if expression exists
        if let Some(expr) = &self.expr_opt {
            expr.format(formatted_code, formatter)?;
        }

        write!(formatted_code, "{}", self.semicolon_token.ident().as_str())?;

        rewrite_with_comments::<ItemConst>(formatter, self.span(), formatted_code, last)?;
        Ok(())
    }
}

impl LeafSpans for ItemConst {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        if let Some(visibility) = &self.visibility {
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
