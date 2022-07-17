use crate::{
    fmt::*,
    utils::bracket::{Parenthesis, SquareBracket},
};
use std::fmt::Write;
use sway_parse::{ExprArrayDescriptor, ExprTupleDescriptor};
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
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        Ok(())
    }
    fn close_parenthesis(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
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
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        Ok(())
    }
    fn close_square_bracket(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        Ok(())
    }
}
