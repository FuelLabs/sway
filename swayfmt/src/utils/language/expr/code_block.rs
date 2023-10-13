use crate::{
    comments::write_comments,
    config::items::ItemBraceStyle,
    formatter::{shape::LineStyle, *},
    utils::{
        map::byte_span::{ByteSpan, LeafSpans},
        CurlyBrace,
    },
};
use std::fmt::Write;
use sway_ast::CodeBlockContents;
use sway_types::{ast::Delimiter, Spanned};

impl Format for CodeBlockContents {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        if !self.statements.is_empty() || self.final_expr_opt.is_some() {
            match formatter.shape.code_line.line_style {
                LineStyle::Inline => {
                    write!(formatted_code, " ")?;
                    for statement in self.statements.iter() {
                        statement.format(formatted_code, formatter)?;
                    }
                    if let Some(final_expr) = &self.final_expr_opt {
                        final_expr.format(formatted_code, formatter)?;
                    }
                    write!(formatted_code, " ")?;
                }
                _ => {
                    writeln!(formatted_code)?;
                    for statement in self.statements.iter() {
                        write!(formatted_code, "{}", formatter.indent_to_str()?)?;
                        statement.format(formatted_code, formatter)?;
                        if !formatted_code.ends_with('\n') {
                            writeln!(formatted_code)?;
                        }
                    }
                    if let Some(final_expr) = &self.final_expr_opt {
                        write!(formatted_code, "{}", formatter.indent_to_str()?)?;
                        final_expr.format(formatted_code, formatter)?;
                        writeln!(formatted_code)?;
                    }
                }
            }
        } else {
            let comments: bool = write_comments(
                formatted_code,
                self.span().start()..self.span().end(),
                formatter,
            )?;
            if !comments {
                formatter.shape.block_unindent(&formatter.config);
            }
        }

        Ok(())
    }
}

impl CurlyBrace for CodeBlockContents {
    fn open_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        formatter.indent();

        let brace_style = formatter.config.items.item_brace_style;
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add opening brace to the next line.
                write!(line, "\n{}", Delimiter::Brace.as_open_char())?;
            }
            _ => {
                // Add opening brace to the same line
                write!(line, "{}", Delimiter::Brace.as_open_char())?;
            }
        }

        Ok(())
    }
    fn close_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // Unindent by one block
        formatter.unindent();
        write!(
            line,
            "{}{}",
            formatter.indent_to_str()?,
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
