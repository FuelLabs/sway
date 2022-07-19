use crate::{fmt::*, utils::bracket::SquareBracket};
use std::fmt::Write;
use sway_parse::{expr::ReassignmentOp, Assignable, Expr};
use sway_types::Spanned;

impl Format for Assignable {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Assignable::Var(name) => name.format(formatted_code, formatter)?,
            Assignable::Index { target, arg } => {
                target.format(formatted_code, formatter)?;
                Expr::open_square_bracket(formatted_code, formatter)?;
                arg.clone().into_inner().format(formatted_code, formatter)?;
                Expr::close_square_bracket(formatted_code, formatter)?;
            }
            Assignable::FieldProjection {
                target,
                dot_token,
                name,
            } => {
                target.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", dot_token.span().as_str())?;
                name.format(formatted_code, formatter)?;
            }
            Assignable::TupleFieldProjection {
                target,
                dot_token,
                field: _,
                field_span,
            } => {
                target.format(formatted_code, formatter)?;
                write!(
                    formatted_code,
                    "{}{}",
                    dot_token.span().as_str(),
                    field_span.as_str()
                )?;
            }
        }
        Ok(())
    }
}

impl Format for ReassignmentOp {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(formatted_code, "{}", self.span.as_str())?;
        Ok(())
    }
}
