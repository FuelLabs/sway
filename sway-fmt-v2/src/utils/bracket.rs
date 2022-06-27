//! The purpose of this file is to house the traits and associated functions for formatting opening and closing delimiters.
//! This allows us to avoid matching a second time for the `ItemKind` and keeps the code pertaining to individual formatting
//! contained to each item's file.
use crate::{Formatter, FormatterError};

pub trait CurlyBrace {
    /// Handles brace open scenerio. Checks the config for the placement of the brace.
    /// Modifies the current shape of the formatter.
    fn open_curly_brace(line: &mut String, formatter: &mut Formatter)
        -> Result<(), FormatterError>;

    /// Handles brace close scenerio.
    /// Currently it simply pushes a `}` and modifies the shape.
    fn close_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError>;
}

pub trait SquareBracket {
    fn open_square_bracket(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError>;

    fn close_square_bracket(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError>;
}

pub trait Parenthesis {
    /// Handles open parenthesis scenarios, checking the config for placement
    /// and modifying the shape of the formatter where necessary.
    fn open_parenthesis(line: &mut String, formatter: &mut Formatter)
        -> Result<(), FormatterError>;

    /// Handles the closing parenthesis scenario.
    fn close_parenthesis(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError>;
}

pub trait AngleBracket {
    fn open_angle_bracket(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError>;

    fn close_angle_bracket(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError>;
}
