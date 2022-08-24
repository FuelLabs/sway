use crate::{
    config::items::ItemBraceStyle,
    formatter::*,
    utils::{
        map::byte_span::{ByteSpan, LeafSpans},
        CurlyBrace,
    },
};
use std::fmt::Write;
use sway_ast::{token::Delimiter, CodeBlockContents};

impl Format for CodeBlockContents {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        writeln!(formatted_code)?;
        for statement in self.statements.iter() {
            statement.format(formatted_code, formatter)?;
        }
        if let Some(final_expr) = &self.final_expr_opt {
            write!(
                formatted_code,
                "{}",
                formatter.shape.indent.to_string(&formatter.config)?
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
        formatter.shape.block_indent(&formatter.config);

        let brace_style = formatter.config.items.item_brace_style;
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add openning brace to the next line.
                write!(line, "\n{}", Delimiter::Brace.as_open_char())?;
            }
            _ => {
                // Add opening brace to the same line
                write!(line, " {}", Delimiter::Brace.as_open_char())?;
            }
        }

        Ok(())
    }
    fn close_curly_brace(
        line: &mut FormattedCode,
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

impl LeafSpans for CodeBlockContents {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_span = Vec::new();
        for statement in self.statements.iter() {
            collected_span.append(&mut statement.leaf_spans());
        }
        if let Some(expr) = &self.final_expr_opt {
            collected_span.append(&mut expr.leaf_spans());
        }
        collected_span
    }
}
