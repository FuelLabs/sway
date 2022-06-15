use crate::Formatter;

pub trait CurlyDelimiter {
    /// Handles brace open scenerio. Checks the config for the placement of the brace.
    /// Modifies the current shape of the formatter.
    fn handle_open_brace(push_to: &mut String, formatter: &mut Formatter);

    /// Handles brace close scenerio.
    /// Currently it simply pushes a `}` and modifies the shape.
    fn handle_closed_brace(push_to: &mut String, formatter: &mut Formatter);
}

pub trait Parenthesis {
    fn open_parenthesis(line: &mut String, formatter: &mut Formatter);

    fn close_parenthesis(line: &mut String, formatter: &mut Formatter);
}
