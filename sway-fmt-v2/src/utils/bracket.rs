//! The purpose of this file is to house the traits and associated functions for formatting opening and closing delimiters.
//! This allows us to avoid matching a second time for the `ItemKind` and keeps the code pertaining to individual formatting
//! contained to each item's file.
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
