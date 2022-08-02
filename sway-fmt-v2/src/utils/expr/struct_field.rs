use crate::{
    config::items::ItemBraceStyle,
    fmt::*,
    utils::{
        bracket::CurlyBrace,
        comments::{ByteSpan, LeafSpans},
    },
};
use std::fmt::Write;
use sway_parse::{token::Delimiter, ExprStructField};
use sway_types::Spanned;

impl Format for ExprStructField {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(
            formatted_code,
            "{}{}",
            formatter.shape.indent.to_string(&formatter.config)?,
            self.field_name.span().as_str()
        )?;
        if let Some(expr) = &self.expr_opt {
            write!(formatted_code, "{} ", expr.0.span().as_str())?;
            expr.1.format(formatted_code, formatter)?;
        }
        // writeln!(formatted_code)?;

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
        write!(
            line,
            "{}{}",
            formatter.shape.indent.to_string(&formatter.config)?,
            Delimiter::Brace.as_close_char()
        )?;
        Ok(())
    }
}

impl LeafSpans for ExprStructField {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.field_name.span())];
        if let Some(expr) = &self.expr_opt {
            collected_spans.push(ByteSpan::from(expr.0.span()));
            // TODO: determine if we are allowing comments between `:` and expr
            collected_spans.append(&mut expr.1.leaf_spans());
        }
        collected_spans
    }
}
