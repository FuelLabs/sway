use crate::{
    config::items::ItemBraceStyle,
    fmt::*,
    utils::{
        bracket::CurlyBrace,
        comments::{ByteSpan, LeafSpans},
        shape::LineStyle,
    },
};
use std::fmt::Write;
use sway_ast::{token::Delimiter, ExprStructField};
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

impl CurlyBrace for ExprStructField {
    fn open_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let brace_style = formatter.config.items.item_brace_style;
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add openning brace to the next line.
                write!(line, "\n{}", Delimiter::Brace.as_open_char())?;
                formatter.shape.block_indent(&formatter.config);
            }
            _ => {
                // Add opening brace to the same line
                write!(line, " {}", Delimiter::Brace.as_open_char())?;
                formatter.shape.block_indent(&formatter.config);
            }
        }

        Ok(())
    }

    fn close_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // Unindent by one block
        formatter.shape.block_unindent(&formatter.config);
        match formatter.shape.code_line.line_style {
            LineStyle::Inline => write!(line, "{}", Delimiter::Brace.as_close_char())?,
            _ => write!(
                line,
                "{}{}",
                formatter.shape.indent.to_string(&formatter.config)?,
                Delimiter::Brace.as_close_char()
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
