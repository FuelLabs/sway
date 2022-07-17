use crate::{config::items::ItemBraceStyle, fmt::*, utils::bracket::CurlyBrace};
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
            formatter.shape.indent.to_string(formatter),
            self.field_name.span().as_str()
        )?;
        if let Some(expr) = &self.expr_opt {
            write!(formatted_code, "{} ", expr.0.span().as_str())?;
            expr.1.format(formatted_code, formatter)?;
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
        let extra_width = formatter.config.whitespace.tab_spaces;
        let mut shape = formatter.shape;
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add openning brace to the next line.
                write!(line, "\n{}", Delimiter::Brace.as_open_char())?;
                shape = shape.block_indent(extra_width);
            }
            _ => {
                // Add opening brace to the same line
                write!(line, " {}", Delimiter::Brace.as_open_char())?;
                shape = shape.block_indent(extra_width);
            }
        }

        formatter.shape = shape;
        Ok(())
    }

    fn close_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // Unindent by one block
        formatter.shape.indent = formatter.shape.indent.block_unindent(formatter);
        write!(
            line,
            "{}{}",
            formatter.shape.indent.to_string(formatter),
            Delimiter::Brace.as_close_char()
        )?;
        Ok(())
    }
}
