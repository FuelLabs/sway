use crate::Formatter;

pub trait CurlyDelimiter {
    /// Handles bracket open scenerio. Checks the config for the placement of the bracket.
    /// Modifies the current shape of the formatter.
    fn handle_open_bracket(push_to: &mut String, formatter: &mut Formatter);

    /// Handles bracket close scenerio.
    /// Currently it simply pushes a `}` and modifies the shape.
    fn handle_closed_bracket(push_to: &mut String, formatter: &mut Formatter);
}
