use crate::{config::items::ItemBraceStyle, fmt::*, utils::bracket::CurlyBrace};
use std::fmt::Write;
use sway_parse::{token::Delimiter, CodeBlockContents};

impl Format for CodeBlockContents {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        for statement in self.statements.iter() {
            statement.format(formatted_code, formatter)?;
        }
        if let Some(final_expr) = &self.final_expr_opt {
            write!(
                formatted_code,
                "{}",
                formatter.shape.indent.to_string(formatter)
            )?;
            final_expr.format(formatted_code, formatter)?;
            writeln!(formatted_code)?;
        }

        Ok(())
    }
}

impl CurlyBrace for CodeBlockContents {
    fn open_curly_brace(
        line: &mut FormattedCode,
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
        line: &mut FormattedCode,
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
