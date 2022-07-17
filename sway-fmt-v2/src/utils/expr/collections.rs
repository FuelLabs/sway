use crate::{
    fmt::*,
    utils::bracket::{Parenthesis, SquareBracket},
};
// use std::fmt::Write;
use sway_parse::{ExprArrayDescriptor, ExprTupleDescriptor};

impl Format for ExprTupleDescriptor {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
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
