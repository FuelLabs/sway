use crate::{
    config::items::ItemBraceStyle,
    formatter::{
        shape::{ExprKind, LineStyle},
        *,
    },
    utils::{
        map::byte_span::{ByteSpan, LeafSpans},
        FormatCurlyBrace,
    },
};
use std::fmt::Write;
use sway_ast::{Braces, CodeBlockContents};
use sway_types::Spanned;

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
                        write!(
                            formatted_code,
                            "{}",
                            formatter.shape.indent.to_string(&formatter.config)?
                        )?;
                        statement.format(formatted_code, formatter)?;
                        if !formatted_code.ends_with('\n') {
                            writeln!(formatted_code)?;
                        }
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
                }
            }
        }

        Ok(())
    }
}

impl FormatCurlyBrace for Braces<CodeBlockContents> {
    fn open_curly_brace(
        &self,
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let open_brace = self.open_token.span().as_str();
        let brace_style = formatter.config.items.item_brace_style;
        if let ExprKind::Conditional = formatter.shape.code_line.expr_kind {
            match formatter.shape.code_line.line_style {
                LineStyle::Multiline => {
                    formatter.shape.code_line.reset_width();
                    write!(
                        line,
                        "\n{}{open_brace}",
                        formatter.shape.indent.to_string(&formatter.config)?
                    )?;
                    formatter
                        .shape
                        .code_line
                        .update_line_style(LineStyle::Normal);
                }
                _ => {
                    write!(line, " {open_brace}")?;
                }
            }
            formatter.shape.block_indent(&formatter.config);
        } else {
            formatter.shape.block_indent(&formatter.config);
            match brace_style {
                ItemBraceStyle::AlwaysNextLine => {
                    // Add openning brace to the next line.
                    write!(line, "\n{open_brace}")?;
                }
                _ => {
                    // Add opening brace to the same line
                    write!(line, " {open_brace}")?;
                }
            }
        }

        Ok(())
    }
    fn close_curly_brace(
        &self,
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let close_brace = self.close_token.span().as_str();
        if let ExprKind::Conditional = formatter.shape.code_line.expr_kind {
            formatter.shape.block_unindent(&formatter.config);
            match formatter.shape.code_line.line_style {
                LineStyle::Inline => {
                    write!(line, "{close_brace}")?;
                }
                _ => {
                    write!(
                        line,
                        "{}{close_brace}",
                        formatter.shape.indent.to_string(&formatter.config)?,
                    )?;
                }
            }
        } else if let ExprKind::MatchBranchKind = formatter.shape.code_line.expr_kind {
            write!(line, "{close_brace}")?;
        } else {
            formatter.shape.block_unindent(&formatter.config);
            write!(
                line,
                "{}{close_brace}",
                formatter.shape.indent.to_string(&formatter.config)?,
            )?;
        }

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
