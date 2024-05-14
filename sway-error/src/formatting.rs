//! This module contains various helper functions for easier formatting and creation of user-friendly
//! diagnostic messages.

use std::{
    borrow::Cow,
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

/// Returns the suffix of the `call_path` together with any type arguments if they
/// exist.
/// Convenient for subsequent showing of only the short name of a full name that was
/// already shown.
///
/// E.g.:
/// SomeName -> SomeName
/// SomeName<T> -> SomeName<T>
/// std::ops::Eq -> Eq
/// some_lib::Struct<A, B> -> Struct<A, B>
pub(crate) fn call_path_suffix_with_args(call_path: &String) -> Cow<String> {
    match call_path.rfind(':') {
        Some(index) if index < call_path.len() - 1 => {
            Cow::Owned(call_path.split_at(index + 1).1.to_string())
        }
        _ => Cow::Borrowed(call_path),
    }
}

/// Returns indefinite article "a" or "an" that corresponds to the `word`,
/// or an empty string if the indefinite article do not fit to the word.
///
/// Note that the function does not recognize plurals and assumes that the
/// `word` is in singular.
///
/// If an article is returned, it is followed by a space, e.g. "a ".
pub(crate) fn a_or_an(word: &'static str) -> &'static str {
    let is_a = in_definite::is_an(word);
    match is_a {
        in_definite::Is::An => "an ",
        in_definite::Is::A => "a ",
        in_definite::Is::None => "",
    }
}

/// Returns `text` with the first character turned into ASCII uppercase.
pub(crate) fn ascii_sentence_case(text: &String) -> Cow<String> {
    if text.is_empty() || text.chars().next().unwrap().is_uppercase() {
        Cow::Borrowed(text)
    } else {
        let mut result = text.clone();
        result[0..1].make_ascii_uppercase();
        Cow::Owned(result.to_owned())
    }
}

/// Returns the first line in `text`, up to the first `\n` if the `text` contains
/// multiple lines, and optionally adds ellipses "..." to the end of the line
/// if `with_ellipses` is true.
///
/// If the `text` is a single-line string, returns the original `text`.
///
/// Suitable for showing just the first line of a piece of code.
/// E.g., if `text` is:
///   if x {
///     0
///   } else {
///     1
///   }
///  the returned value, with ellipses, will be:
///   if x {...
pub(crate) fn first_line(text: &str, with_ellipses: bool) -> Cow<str> {
    if !text.contains('\n') {
        Cow::Borrowed(text)
    } else {
        let index_of_new_line = text.find('\n').unwrap();
        Cow::Owned(text[..index_of_new_line].to_string() + if with_ellipses { "..." } else { "" })
    }
}
