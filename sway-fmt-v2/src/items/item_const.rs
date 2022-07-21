use crate::{
    fmt::{Format, FormattedCode, Formatter},
    utils::comments::{ByteSpan, CommentVisitor},
    FormatterError,
};
use std::fmt::Write;
use sway_parse::ItemConst;
use sway_types::Spanned;

impl Format for ItemConst {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // Check if visibility token exists if so add it.
        if let Some(visibility_token) = &self.visibility {
            write!(formatted_code, "{} ", visibility_token.span().as_str())?;
        }

        // Add the const token
        write!(formatted_code, "{} ", self.const_token.span().as_str())?;

        // Add name of the const
        write!(formatted_code, "{}", self.name.as_str())?;

        // Check if ty exists
        if let Some(ty) = &self.ty_opt {
            // Add colon
            write!(formatted_code, "{} ", ty.0.span().as_str())?;
            ty.1.format(formatted_code, formatter)?;
        }

        // ` = `
        write!(formatted_code, " {} ", self.eq_token.ident().as_str())?;

        // TODO: We are not applying any custom formatting to expr, probably we will need to in the future.
        write!(
            formatted_code,
            "{}{}",
            self.expr.span().as_str(),
            self.semicolon_token.ident().as_str()
        )?;

        Ok(())
    }
}

impl CommentVisitor for ItemConst {
    fn collect_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        if let Some(visibility) = &self.visibility {
            collected_spans.push(ByteSpan::from(visibility.span()));
        }
        collected_spans.push(ByteSpan::from(self.const_token.span()));
        collected_spans.push(ByteSpan::from(self.name.span()));
        if let Some(ty) = &self.ty_opt {
            collected_spans.append(&mut ty.collect_spans());
            // TODO: determine if we allow comments in between `:` and ty
        }
        collected_spans.push(ByteSpan::from(self.eq_token.span()));
        collected_spans.append(&mut self.expr.collect_spans());
        collected_spans.push(ByteSpan::from(self.semicolon_token.span()));
        collected_spans
    }
}
