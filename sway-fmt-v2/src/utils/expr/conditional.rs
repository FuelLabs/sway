use crate::{
    fmt::*,
    utils::{
        bracket::CurlyBrace,
        byte_span::{ByteSpan, LeafSpans},
        shape::{ExprKind, LineStyle},
    },
};
use std::{fmt::Write, ops::ControlFlow};
use sway_ast::{token::Delimiter, IfCondition, IfExpr, MatchBranch, MatchBranchKind};
use sway_types::Spanned;

use super::debug_expr;

impl Format for IfExpr {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        formatter
            .shape
            .code_line
            .update_expr_kind(ExprKind::Conditional);
        // check if the entire expression could fit into a single line
        let full_width_line_style = get_full_width_line_style(self, formatter)?;
        if full_width_line_style == LineStyle::Inline {
            formatter
                .shape
                .code_line
                .update_line_style(full_width_line_style);
            format_if_expr(self, formatted_code, formatter)?;
        } else {
            // if it can't then we must format one expression at a time
            let if_cond_width = get_if_condition_width(self)?;
            formatter
                .shape // add an offest for now of 5, that represents a possible `else`
                .get_line_style(None, Some(if_cond_width), &formatter.config);
            format_if_condition(self, formatted_code, formatter)?;
            println!(
                "2nd line: \"{}\"\nwidth: {}\n{:?}",
                formatted_code, formatter.shape.width, formatter.shape.code_line.line_style
            );
            format_then_block(self, formatted_code, formatter)?;

            if self.else_opt.is_some() {
                format_else_opt(self, formatted_code, formatter)?;
            }
        }

        Ok(())
    }
}

fn get_full_width_line_style(
    if_expr: &IfExpr,
    formatter: &mut Formatter,
) -> Result<LineStyle, FormatterError> {
    let mut temp_formatter = Formatter::default();
    temp_formatter
        .shape
        .code_line
        .update_expr_kind(ExprKind::Conditional);
    temp_formatter
        .shape
        .code_line
        .update_line_style(LineStyle::Inline);

    let mut if_expr_str = FormattedCode::new();
    format_if_expr(if_expr, &mut if_expr_str, &mut temp_formatter)?;
    let if_expr_width = if_expr_str.chars().count();

    temp_formatter.shape.update_width(if_expr_width);
    formatter.shape.update_width(if_expr_width);
    temp_formatter
        .shape
        .get_line_style(None, None, &temp_formatter.config);

    Ok(temp_formatter.shape.code_line.line_style)
}

fn get_if_condition_width(if_expr: &IfExpr) -> Result<usize, FormatterError> {
    let mut temp_formatter = Formatter::default();
    temp_formatter
        .shape
        .code_line
        .update_expr_kind(ExprKind::Conditional);

    let mut if_cond_str = FormattedCode::new();
    format_if_condition(if_expr, &mut if_cond_str, &mut temp_formatter)?;
    write!(if_cond_str, " {{")?;
    let condition_width = if_cond_str.chars().count();

    debug_expr(
        if_cond_str,
        None,
        None,
        condition_width,
        &mut temp_formatter,
    );

    Ok(condition_width)
}

fn format_if_expr(
    if_expr: &IfExpr,
    formatted_code: &mut FormattedCode,
    formatter: &mut Formatter,
) -> Result<(), FormatterError> {
    format_if_condition(if_expr, formatted_code, formatter)?;
    format_then_block(if_expr, formatted_code, formatter)?;
    format_else_opt(if_expr, formatted_code, formatter)?;

    Ok(())
}

fn format_if_condition(
    if_expr: &IfExpr,
    formatted_code: &mut FormattedCode,
    formatter: &mut Formatter,
) -> Result<(), FormatterError> {
    write!(formatted_code, "{} ", if_expr.if_token.span().as_str())?;
    if formatter.shape.code_line.line_style == LineStyle::Multiline {
        formatter.shape.block_indent(&formatter.config);
        if_expr.condition.format(formatted_code, formatter)?;
        formatter.shape.block_unindent(&formatter.config);
    } else {
        if_expr.condition.format(formatted_code, formatter)?;
    }

    Ok(())
}

fn format_then_block(
    if_expr: &IfExpr,
    formatted_code: &mut FormattedCode,
    formatter: &mut Formatter,
) -> Result<(), FormatterError> {
    IfExpr::open_curly_brace(formatted_code, formatter)?;
    if_expr.then_block.get().format(formatted_code, formatter)?;
    if if_expr.else_opt.is_none() {
        IfExpr::close_curly_brace(formatted_code, formatter)?;
    }

    Ok(())
}

fn format_else_opt(
    if_expr: &IfExpr,
    formatted_code: &mut FormattedCode,
    formatter: &mut Formatter,
) -> Result<(), FormatterError> {
    if let Some((else_token, control_flow)) = &if_expr.else_opt {
        let mut else_if_str = FormattedCode::new();

        IfExpr::close_curly_brace(&mut else_if_str, formatter)?;
        write!(else_if_str, " {}", else_token.span().as_str())?;
        match &control_flow {
            ControlFlow::Continue(if_expr) => {
                write!(else_if_str, " ")?;
                if_expr.format(&mut else_if_str, formatter)?
            }
            ControlFlow::Break(code_block_contents) => {
                IfExpr::open_curly_brace(&mut else_if_str, formatter)?;
                code_block_contents
                    .get()
                    .format(&mut else_if_str, formatter)?;
                IfExpr::close_curly_brace(&mut else_if_str, formatter)?;
            }
        }

        write!(formatted_code, "{else_if_str}")?;
    }

    Ok(())
}

impl CurlyBrace for IfExpr {
    fn open_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let open_brace = Delimiter::Brace.as_open_char();
        match formatter.shape.code_line.line_style {
            LineStyle::Multiline => {
                formatter.shape.reset_width();
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

        Ok(())
    }
    fn close_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
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

impl Format for IfCondition {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Self::Expr(expr) => {
                expr.format(formatted_code, formatter)?;
            }
            Self::Let {
                let_token,
                lhs,
                eq_token,
                rhs,
            } => {
                write!(formatted_code, "{} ", let_token.span().as_str())?;
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", eq_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
        }

        Ok(())
    }
}

impl Format for MatchBranch {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        self.pattern.format(formatted_code, formatter)?;
        write!(
            formatted_code,
            " {} ",
            self.fat_right_arrow_token.span().as_str()
        )?;
        self.kind.format(formatted_code, formatter)?;

        Ok(())
    }
}

impl CurlyBrace for MatchBranch {
    fn open_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        formatter.shape.block_indent(&formatter.config);
        writeln!(line, "{}", Delimiter::Brace.as_open_char())?;

        Ok(())
    }
    fn close_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
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

// Later we should add logic to handle transforming `Block` -> `Expr` and vice versa.
impl Format for MatchBranchKind {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Self::Block {
                block,
                comma_token_opt,
            } => {
                Self::open_curly_brace(formatted_code, formatter)?;
                let block = block.get();
                if block.statements.is_empty() && block.final_expr_opt.is_none() {
                    // even if there is no code block we still want to unindent
                    // before the closing brace
                    formatter.shape.block_unindent(&formatter.config);
                } else {
                    block.format(formatted_code, formatter)?;
                    // we handle this here to avoid needless indents
                    formatter.shape.block_unindent(&formatter.config);
                    write!(
                        formatted_code,
                        "{}",
                        formatter.shape.indent.to_string(&formatter.config)?
                    )?;
                }
                Self::close_curly_brace(formatted_code, formatter)?;
                if let Some(comma_token) = comma_token_opt {
                    write!(formatted_code, "{}", comma_token.span().as_str())?;
                }
            }
            Self::Expr { expr, comma_token } => {
                expr.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", comma_token.span().as_str())?;
            }
        }

        Ok(())
    }
}

impl CurlyBrace for MatchBranchKind {
    fn open_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        formatter.shape.block_indent(&formatter.config);
        write!(line, "{}", Delimiter::Brace.as_open_char())?;
        Ok(())
    }
    fn close_curly_brace(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Brace.as_close_char())?;
        Ok(())
    }
}

impl LeafSpans for IfExpr {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.if_token.span())];
        collected_spans.append(&mut self.condition.leaf_spans());
        collected_spans.append(&mut self.then_block.leaf_spans());
        if let Some(else_block) = &self.else_opt {
            collected_spans.push(ByteSpan::from(else_block.0.span()));
            let mut else_body_spans = match &else_block.1 {
                std::ops::ControlFlow::Continue(if_expr) => if_expr.leaf_spans(),
                std::ops::ControlFlow::Break(else_body) => else_body.leaf_spans(),
            };
            collected_spans.append(&mut else_body_spans);
        }
        collected_spans
    }
}

impl LeafSpans for IfCondition {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        match self {
            IfCondition::Expr(expr) => expr.leaf_spans(),
            IfCondition::Let {
                let_token,
                lhs,
                eq_token,
                rhs,
            } => {
                let mut collected_spans = vec![ByteSpan::from(let_token.span())];
                collected_spans.append(&mut lhs.leaf_spans());
                collected_spans.push(ByteSpan::from(eq_token.span()));
                collected_spans.append(&mut rhs.leaf_spans());
                collected_spans
            }
        }
    }
}

impl LeafSpans for MatchBranch {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        collected_spans.append(&mut self.pattern.leaf_spans());
        collected_spans.push(ByteSpan::from(self.fat_right_arrow_token.span()));
        collected_spans.append(&mut self.kind.leaf_spans());
        collected_spans
    }
}

impl LeafSpans for MatchBranchKind {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        match self {
            MatchBranchKind::Block {
                block,
                comma_token_opt,
            } => {
                collected_spans.append(&mut block.leaf_spans());
                if let Some(comma_token) = comma_token_opt {
                    collected_spans.push(ByteSpan::from(comma_token.span()));
                }
            }
            MatchBranchKind::Expr { expr, comma_token } => {
                collected_spans.append(&mut expr.leaf_spans());
                collected_spans.push(ByteSpan::from(comma_token.span()));
            }
        };
        collected_spans
    }
}
