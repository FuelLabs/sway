use crate::{
    config::items::ItemBraceStyle,
    formatter::{shape::LineStyle, *},
    utils::{
        map::byte_span::{ByteSpan, LeafSpans},
        FormatCurlyBrace,
    },
};
use std::fmt::Write;
use sway_ast::{Braces, CommaToken, ExprStructField, Punctuated};
use sway_types::Spanned;

impl Format for ExprStructField {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(formatted_code, "{}", self.field_name.span().as_str())?;
        if let Some((colon_token, expr)) = &self.expr_opt {
            write!(formatted_code, "{} ", colon_token.span().as_str())?;
            expr.format(formatted_code, formatter)?;
        }

        Ok(())
    }
}

impl FormatCurlyBrace for Braces<Punctuated<ExprStructField, CommaToken>> {
    fn open_curly_brace(
        &self,
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let brace_style = formatter.config.items.item_brace_style;
        let open_brace = self.open_token.span().as_str();
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add openning brace to the next line.
                write!(line, "\n{open_brace}")?;
                formatter.shape.block_indent(&formatter.config);
            }
            _ => {
                // Add opening brace to the same line
                write!(line, " {open_brace}")?;
                formatter.shape.block_indent(&formatter.config);
            }
        }

        Ok(())
    }

    fn close_curly_brace(
        &self,
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let close_brace = self.close_token.span().as_str();
        // Unindent by one block
        formatter.shape.block_unindent(&formatter.config);
        match formatter.shape.code_line.line_style {
            LineStyle::Inline => write!(line, "{close_brace}")?,
            _ => write!(
                line,
                "{}{close_brace}",
                formatter.shape.indent.to_string(&formatter.config)?,
            )?,
        }

        Ok(())
    }
}

impl LeafSpans for ExprStructField {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.field_name.span())];
        if let Some((colon_token, expr)) = &self.expr_opt {
            collected_spans.push(ByteSpan::from(colon_token.span()));
            collected_spans.append(&mut expr.leaf_spans());
        }
        collected_spans
    }
}
