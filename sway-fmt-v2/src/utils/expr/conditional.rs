use crate::{
    fmt::*,
    utils::{
        bracket::CurlyBrace,
        byte_span::{ByteSpan, LeafSpans},
    },
};
use std::{fmt::Write, ops::ControlFlow};
use sway_ast::{token::Delimiter, IfCondition, IfExpr, MatchBranch, MatchBranchKind};
use sway_types::Spanned;

impl Format for IfExpr {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(formatted_code, "{} ", self.if_token.span().as_str())?;
        self.condition.format(formatted_code, formatter)?;
        Self::open_curly_brace(formatted_code, formatter)?;
        self.then_block.get().format(formatted_code, formatter)?;
        Self::close_curly_brace(formatted_code, formatter)?;
        if let Some(else_opt) = &self.else_opt {
            write!(formatted_code, "{} ", else_opt.0.span().as_str())?;
            match &else_opt.1 {
                ControlFlow::Continue(if_expr) => if_expr.format(formatted_code, formatter)?,
                ControlFlow::Break(code_block_contents) => {
                    Self::open_curly_brace(formatted_code, formatter)?;
                    code_block_contents
                        .get()
                        .format(formatted_code, formatter)?;
                    Self::close_curly_brace(formatted_code, formatter)?;
                }
            }
        }

        Ok(())
    }
}

impl CurlyBrace for IfExpr {
    fn open_curly_brace(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
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
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
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
                block.get().format(formatted_code, formatter)?;
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
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
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
