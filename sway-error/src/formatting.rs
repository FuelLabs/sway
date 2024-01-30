//! This module contains various helper functions for easier formatting and creation of user-friendly
//! diagnostic messages.

use std::{
    cmp,
    fmt::{self, Display},
};

use sway_types::{SourceEngine, SourceId};

/// Returns the file name (with extension) for the provided `source_id`,
/// or `None` if the `source_id` is `None` or the file name cannot be
/// obtained.
pub(crate) fn get_file_name(
    source_engine: &SourceEngine,
    source_id: Option<&SourceId>,
) -> Option<String> {
    match source_id {
        Some(source_id) => source_engine.get_file_name(source_id),
        None => None,
    }
}

/// Returns reading-friendly textual representation for `number` smaller than or equal to 10
/// or its numeric representation if it is greater than 10.
pub(crate) fn number_to_str(number: usize) -> String {
    match number {
        0 => "zero".to_string(),
        1 => "one".to_string(),
        2 => "two".to_string(),
        3 => "three".to_string(),
        4 => "four".to_string(),
        5 => "five".to_string(),
        6 => "six".to_string(),
        7 => "seven".to_string(),
        8 => "eight".to_string(),
        9 => "nine".to_string(),
        10 => "ten".to_string(),
        _ => format!("{number}"),
    }
}

pub(crate) enum Enclosing {
    #[allow(dead_code)]
    None,
    DoubleQuote,
}

impl Display for Enclosing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "",
                Self::DoubleQuote => "\"",
            },
        )
    }
}

pub(crate) enum Indent {
    #[allow(dead_code)]
    None,
    Single,
    Double,
}

impl Display for Indent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "",
                Self::Single => "  ",
                Self::Double => "    ",
            },
        )
    }
}

/// Returns reading-friendly textual representation of the `sequence`, with comma-separated
/// items and each item optionally enclosed in the specified `enclosing`.
/// If the sequence has more than `max_items` the remaining items are replaced
/// with the text "and <number> more".
///
/// E.g.:
/// [a] => "a"
/// [a, b] => "a" and "b"
/// [a, b, c] => "a", "b" and "c"
/// [a, b, c, d] => "a", "b", "c" and one more
/// [a, b, c, d, e] => "a", "b", "c" and two more
///
/// Panics if the `sequence` is empty, or `max_items` is zero.
pub(crate) fn sequence_to_str<T>(sequence: &[T], enclosing: Enclosing, max_items: usize) -> String
where
    T: Display,
{
    assert!(
        !sequence.is_empty(),
        "Sequence to display must not be empty."
    );
    assert!(
        max_items > 0,
        "Maximum number of items to display must be greater than zero."
    );

    let max_items = cmp::min(max_items, sequence.len());

    let (to_display, remaining) = sequence.split_at(max_items);

    let fmt_item = |item: &T| format!("{enclosing}{item}{enclosing}");

    if !remaining.is_empty() {
        format!(
            "{}, and {} more",
            to_display
                .iter()
                .map(fmt_item)
                .collect::<Vec<_>>()
                .join(", "),
            number_to_str(remaining.len())
        )
    } else {
        match to_display {
            [] => unreachable!("There must be at least one item in the sequence."),
            [item] => fmt_item(item),
            [first_item, second_item] => {
                format!("{} and {}", fmt_item(first_item), fmt_item(second_item))
            }
            _ => format!(
                "{}, and {}",
                to_display
                    .split_last()
                    .unwrap()
                    .1
                    .iter()
                    .map(fmt_item)
                    .collect::<Vec::<_>>()
                    .join(", "),
                fmt_item(to_display.last().unwrap())
            ),
        }
    }
}

/// Returns reading-friendly textual representation of the `sequence`, with vertically
/// listed items and each item indented for the `indent` and preceded with the dash (-).
/// If the sequence has more than `max_items` the remaining items are replaced
/// with the text "and <number> more".
///
/// E.g.:
/// [a] =>
///   - a
/// [a, b] =>
///   - a
///   - b
/// [a, b, c, d, e] =>
///   - a
///   - b
///   - and three more
///
/// Panics if the `sequence` is empty, or `max_items` is zero.
pub(crate) fn sequence_to_list<T>(sequence: &[T], indent: Indent, max_items: usize) -> Vec<String>
where
    T: Display,
{
    assert!(
        !sequence.is_empty(),
        "Sequence to display must not be empty."
    );
    assert!(
        max_items > 0,
        "Maximum number of items to display must be greater than zero."
    );

    let mut result = vec![];

    let max_items = cmp::min(max_items, sequence.len());
    let (to_display, remaining) = sequence.split_at(max_items);
    for item in to_display {
        result.push(format!("{indent}- {item}"));
    }
    if !remaining.is_empty() {
        result.push(format!(
            "{indent}- and {} more",
            number_to_str(remaining.len())
        ));
    }

    result
}

/// Returns "s" if `count` is different than 1, otherwise empty string.
/// Convenient for building simple plural of words.
pub(crate) fn plural_s(count: usize) -> &'static str {
    if count == 1 {
        ""
    } else {
        "s"
    }
}

/// Returns "is" if `count` is 1, otherwise "are".
pub(crate) fn is_are(count: usize) -> &'static str {
    if count == 1 {
        "is"
    } else {
        "are"
    }
}

/// Returns `singular` if `count` is 1, otherwise `plural`.
pub(crate) fn singular_plural<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 {
        singular
    } else {
        plural
    }
}
