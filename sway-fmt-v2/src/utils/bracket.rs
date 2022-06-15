use crate::Formatter;

pub trait CurlyBrace {
    /// Handles brace open scenerio. Checks the config for the placement of the brace.
    /// Modifies the current shape of the formatter.
    fn open_curly_brace(line: &mut String, formatter: &mut Formatter);

    /// Handles brace close scenerio.
    /// Currently it simply pushes a `}` and modifies the shape.
    fn close_curly_brace(line: &mut String, formatter: &mut Formatter);
}

pub trait SquareBracket {
    fn open_square_bracket(line: &mut String, formatter: &mut Formatter);

    fn close_square_bracket(line: &mut String, formatter: &mut Formatter);
}
