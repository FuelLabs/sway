use crate::{
    fmt::*,
    utils::{
        bracket::{Parenthesis, SquareBracket},
        comments::{ByteSpan, LeafSpans},
    },
};
use std::fmt::Write;
use sway_ast::{token::Delimiter, ExprArrayDescriptor, ExprTupleDescriptor};
use sway_types::Spanned;

// TODO: add long and multiline formatting
impl Format for ExprTupleDescriptor {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Self::Nil => {}
            Self::Cons {
                head,
                comma_token,
                tail,
            } => {
                head.format(formatted_code, formatter)?;
                write!(formatted_code, "{} ", comma_token.span().as_str())?;
                tail.format(formatted_code, formatter)?;
            }
        }
        Ok(())
    }
}

impl Parenthesis for ExprTupleDescriptor {
    fn open_parenthesis(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Parenthesis.as_open_char())?;
        Ok(())
    }
    fn close_parenthesis(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Parenthesis.as_close_char())?;
        Ok(())
    }
}

impl Format for ExprArrayDescriptor {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Self::Sequence(punct_expr) => {
                punct_expr.format(formatted_code, formatter)?;
            }
            Self::Repeat {
                value,
                semicolon_token,
                length,
            } => {
                value.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", semicolon_token.span().as_str())?;
                length.format(formatted_code, formatter)?;
            }
        }
        Ok(())
    }
}

impl SquareBracket for ExprArrayDescriptor {
    fn open_square_bracket(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Bracket.as_open_char())?;
        Ok(())
    }
    fn close_square_bracket(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Bracket.as_close_char())?;
        Ok(())
    }
}

impl LeafSpans for ExprTupleDescriptor {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        if let ExprTupleDescriptor::Cons {
            head,
            comma_token,
            tail,
        } = self
        {
            collected_spans.append(&mut head.leaf_spans());
            collected_spans.push(ByteSpan::from(comma_token.span()));
            collected_spans.append(&mut tail.leaf_spans());
        }
        collected_spans
    }
}

impl LeafSpans for ExprArrayDescriptor {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        if let ExprArrayDescriptor::Repeat {
            value,
            semicolon_token,
            length,
        } = self
        {
            collected_spans.append(&mut value.leaf_spans());
            collected_spans.push(ByteSpan::from(semicolon_token.span()));
            collected_spans.append(&mut length.leaf_spans());
        }
        collected_spans
    }
}
