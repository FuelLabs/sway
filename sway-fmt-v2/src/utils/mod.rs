use crate::formatter::*;
use std::fmt::Write;
use sway_ast::token::PunctKind;

pub(crate) mod language;
pub(crate) mod map;

pub(crate) trait CurlyBrace {
    /// Handles brace open scenerio. Checks the config for the placement of the brace.
    /// Modifies the current shape of the formatter.
    fn open_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError>;

    /// Handles brace close scenerio.
    /// Currently it simply pushes a `}` and modifies the shape.
    fn close_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError>;
}

pub(crate) trait SquareBracket {
    fn open_square_bracket(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError>;

    fn close_square_bracket(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError>;
}

pub(crate) trait Parenthesis {
    /// Handles open parenthesis scenarios, checking the config for placement
    /// and modifying the shape of the formatter where necessary.
    fn open_parenthesis(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError>;

    /// Handles the closing parenthesis scenario.
    fn close_parenthesis(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError>;
}

pub(crate) fn open_angle_bracket(formatted_code: &mut FormattedCode) -> Result<(), FormatterError> {
    write!(formatted_code, "{}", PunctKind::LessThan.as_char())?;

    Ok(())
}

pub(crate) fn close_angle_bracket(
    formatted_code: &mut FormattedCode,
) -> Result<(), FormatterError> {
    write!(formatted_code, "{}", PunctKind::GreaterThan.as_char())?;

    Ok(())
}
