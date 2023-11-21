use crate::{
    comments::write_comments,
    formatter::{
        shape::{ExprKind, LineStyle},
        *,
    },
    utils::{
        map::byte_span::{ByteSpan, LeafSpans},
        CurlyBrace,
    },
};
use std::{fmt::Write, ops::Range};
use sway_ast::{expr::LoopControlFlow, IfCondition, IfExpr, MatchBranch, MatchBranchKind};
use sway_types::{ast::Delimiter, Spanned};

impl Format for IfExpr {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        formatter.with_shape(
            formatter
                .shape
                .with_code_line_from(LineStyle::default(), ExprKind::Conditional),
            |formatter| -> Result<(), FormatterError> {
                let range: Range<usize> = self.span().into();
                let comments = formatter.comments_context.map.comments_between(&range);
                // check if the entire expression could fit into a single line
                let full_width_line_style = if comments.peekable().peek().is_some() {
                    LineStyle::Multiline
                } else {
                    get_full_width_line_style(self, formatter)?
                };
                if full_width_line_style == LineStyle::Inline && self.else_opt.is_some() {
                    formatter
                        .shape
                        .code_line
                        .update_line_style(full_width_line_style);
                    format_if_expr(self, formatted_code, formatter)?;
                } else {
                    // if it can't then we must format one expression at a time
                    let if_cond_width = get_if_condition_width(self)?;
                    formatter
                        .shape
                        .get_line_style(None, Some(if_cond_width), &formatter.config);
                    if formatter.shape.code_line.line_style == LineStyle::Inline {
                        formatter
                            .shape
                            .code_line
                            .update_line_style(LineStyle::Normal)
                    }
                    format_if_condition(self, formatted_code, formatter)?;
                    format_then_block(self, formatted_code, formatter)?;

                    if self.else_opt.is_some() {
                        format_else_opt(self, formatted_code, formatter)?;
                    }
                }

                Ok(())
            },
        )?;

        Ok(())
    }
}

fn get_full_width_line_style(
    if_expr: &IfExpr,
    formatter: &mut Formatter,
) -> Result<LineStyle, FormatterError> {
    let mut temp_formatter = Formatter::default();
    let line_style = temp_formatter.with_shape(
        temp_formatter
            .shape
            .with_code_line_from(LineStyle::Inline, ExprKind::Conditional),
        |temp_formatter| -> Result<LineStyle, FormatterError> {
            let mut if_expr_str = FormattedCode::new();
            format_if_expr(if_expr, &mut if_expr_str, temp_formatter)?;
            let if_expr_width = if_expr_str.chars().count();

            temp_formatter.shape.code_line.update_width(if_expr_width);
            formatter.shape.code_line.update_width(if_expr_width);
            temp_formatter
                .shape
                .get_line_style(None, None, &temp_formatter.config);

            Ok(temp_formatter.shape.code_line.line_style)
        },
    )?;

    Ok(line_style)
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
        formatter.indent();
        if_expr.condition.format(formatted_code, formatter)?;
        formatter.unindent();
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

    if !if_expr.then_block.get().statements.is_empty()
        || if_expr.then_block.get().final_expr_opt.is_some()
    {
        if_expr.then_block.get().format(formatted_code, formatter)?;
    } else {
        let comments = write_comments(
            formatted_code,
            if_expr.then_block.span().start()..if_expr.then_block.span().end(),
            formatter,
        )?;
        if !comments {
            formatter.shape.block_unindent(&formatter.config);
        }
    }
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
        let comments_written = write_comments(
            &mut else_if_str,
            if_expr.then_block.span().end()..else_token.span().start(),
            formatter,
        )?;

        if comments_written {
            write!(else_if_str, "{}", formatter.indent_to_str()?,)?;
        } else {
            write!(else_if_str, " ")?;
        }
        write!(else_if_str, "{}", else_token.span().as_str())?;
        match &control_flow {
            LoopControlFlow::Continue(if_expr) => {
                write!(else_if_str, " ")?;
                if_expr.format(&mut else_if_str, formatter)?
            }
            LoopControlFlow::Break(code_block_contents) => {
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
                formatter.shape.code_line.reset_width();
                write!(line, "\n{}{open_brace}", formatter.indent_to_str()?)?;
                formatter
                    .shape
                    .code_line
                    .update_line_style(LineStyle::Normal);
            }
            _ => {
                write!(line, " {open_brace}")?;
            }
        }
        formatter.indent();

        Ok(())
    }
    fn close_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        formatter.unindent();
        match formatter.shape.code_line.line_style {
            LineStyle::Inline => {
                write!(line, "{}", Delimiter::Brace.as_close_char())?;
            }
            _ => {
                write!(
                    line,
                    "{}{}",
                    formatter.indent_to_str()?,
                    Delimiter::Brace.as_close_char()
                )?;
            }
        }

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
        formatter.indent();
        writeln!(line, "{}", Delimiter::Brace.as_open_char())?;

        Ok(())
    }
    fn close_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
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
                    formatter.unindent();
                } else {
                    block.format(formatted_code, formatter)?;
                    // we handle this here to avoid needless indents
                    formatter.unindent();
                    write!(formatted_code, "{}", formatter.indent_to_str()?)?;
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
        formatter.indent();
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
                LoopControlFlow::Continue(if_expr) => if_expr.leaf_spans(),
                LoopControlFlow::Break(else_body) => else_body.leaf_spans(),
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
