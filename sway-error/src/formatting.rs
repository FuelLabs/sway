//! This module contains various helper functions for easier formatting and creation of user-friendly messages.

use std::{
    borrow::Cow,
    cmp::{self, Ordering},
    fmt::{self, Display},
};

use sway_types::{SourceEngine, SourceId, Span};

use crate::diagnostic::Hint;

/// Returns the file name (with extension) for the provided `source_id`,
/// or `None` if the `source_id` is `None` or the file name cannot be
/// obtained.
pub fn get_file_name(source_engine: &SourceEngine, source_id: Option<&SourceId>) -> Option<String> {
    match source_id {
        Some(source_id) => source_engine.get_file_name(source_id),
        None => None,
    }
}

/// Returns reading-friendly textual representation for `num` smaller than or equal to 10
/// or its numeric representation if it is greater than 10.
pub fn num_to_str(num: usize) -> String {
    match num {
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
        _ => format!("{num}"),
    }
}

/// Returns reading-friendly textual representation for `num` smaller than or equal to 10
/// or its numeric representation if it is greater than 10.
///
/// Zero is returned as "none".
pub fn num_to_str_or_none(num: usize) -> String {
    if num == 0 {
        "none".to_string()
    } else {
        num_to_str(num)
    }
}

pub enum Enclosing {
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

pub enum Indent {
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
/// - \[a\] => "a"
/// - \[a, b\] => "a" and "b"
/// - \[a, b, c\] => "a", "b" and "c"
/// - \[a, b, c, d\] => "a", "b", "c" and one more
/// - \[a, b, c, d, e\] => "a", "b", "c" and two more
///
/// Panics if the `sequence` is empty, or `max_items` is zero.
pub fn sequence_to_str<T>(sequence: &[T], enclosing: Enclosing, max_items: usize) -> String
where
    T: Display,
{
    sequence_to_str_impl(sequence, enclosing, max_items, "and")
}

/// Returns reading-friendly textual representation of the `sequence`, with comma-separated
/// items and each item optionally enclosed in the specified `enclosing`.
/// If the sequence has more than `max_items` the remaining items are replaced
/// with the text "or <number> more".
///
/// E.g.:
/// - \[a\] => "a"
/// - \[a, b\] => "a" or "b"
/// - \[a, b, c\] => "a", "b" or "c"
/// - \[a, b, c, d\] => "a", "b", "c" or one more
/// - \[a, b, c, d, e\] => "a", "b", "c" or two more
///
/// Panics if the `sequence` is empty, or `max_items` is zero.
pub fn sequence_to_str_or<T>(sequence: &[T], enclosing: Enclosing, max_items: usize) -> String
where
    T: Display,
{
    sequence_to_str_impl(sequence, enclosing, max_items, "or")
}

fn sequence_to_str_impl<T>(
    sequence: &[T],
    enclosing: Enclosing,
    max_items: usize,
    and_or: &str,
) -> String
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
            "{}, {} {} more",
            to_display
                .iter()
                .map(fmt_item)
                .collect::<Vec<_>>()
                .join(", "),
            and_or,
            num_to_str(remaining.len())
        )
    } else {
        match to_display {
            [] => unreachable!("There must be at least one item in the sequence."),
            [item] => fmt_item(item),
            [first_item, second_item] => {
                format!(
                    "{} {} {}",
                    fmt_item(first_item),
                    and_or,
                    fmt_item(second_item)
                )
            }
            _ => format!(
                "{}, {} {}",
                to_display
                    .split_last()
                    .unwrap()
                    .1
                    .iter()
                    .map(fmt_item)
                    .collect::<Vec::<_>>()
                    .join(", "),
                and_or,
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
/// * \[a\] =>
///     - a
/// * \[a, b\] =>
///     - a
///     - b
/// * \[a, b, c, d, e\] =>
///     - a
///     - b
///     - and three more
///
/// Panics if the `sequence` is empty, or `max_items` is zero.
pub fn sequence_to_list<T>(sequence: &[T], indent: Indent, max_items: usize) -> Vec<String>
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
            num_to_str(remaining.len())
        ));
    }

    result
}

/// Returns "s" if `count` is different than 1, otherwise empty string.
/// Convenient for building simple plural of words.
pub fn plural_s(count: usize) -> &'static str {
    if count == 1 {
        ""
    } else {
        "s"
    }
}

/// Returns "is" if `count` is 1, otherwise "are".
pub fn is_are(count: usize) -> &'static str {
    if count == 1 {
        "is"
    } else {
        "are"
    }
}

/// Returns `singular` if `count` is 1, otherwise `plural`.
pub fn singular_plural<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 {
        singular
    } else {
        plural
    }
}

/// Returns the short name of a type or function represented by the `full_name`.
/// Convenient for subsequent showing only the short name of a full name that was
/// already shown.
///
/// The `full_name` is expected to be a call path with or without generic parameters,
/// eventually prefixed with `&`s or `&mut`s for types.
///
/// E.g.:
/// - `SomeType` -> `SomeType`
/// - `SomeType<T>` -> `SomeType`
/// - `std::ops::Eq` -> `Eq`
/// - `some_lib::Struct<A, B>` -> `Struct`
/// - `some_lib::Struct<some::other::lib::A, some::other::lib::B>` -> `Struct`
/// - `&mut some_lib::Struct<&some::other::lib::A, &mut some::other::lib::B>` -> `&mut Struct`
/// - `&&&mut some_lib::Struct<&some::other::lib::A, &mut some::other::lib::B>` -> `&&&mut Struct`
/// - `some_lib::fns::some_function<A, B>` -> `some_function`
pub fn short_name(full_name: &str) -> String {
    // Preserve leading references, `&`s and `&mut`s.
    let mut name_start_index = 0;
    loop {
        let reminder = &full_name[name_start_index..];
        if reminder.starts_with('&') {
            name_start_index += 1;
        } else if reminder.starts_with("mut ") {
            name_start_index += 4;
        } else {
            break;
        }
    }
    let full_name_without_refs = &full_name[name_start_index..];
    let full_name_without_generics = match full_name_without_refs.find('<') {
        Some(index) => &full_name_without_refs[..index],
        None => full_name_without_refs,
    };
    let short_name = match full_name_without_generics.rfind(':') {
        Some(index) if index < full_name_without_generics.len() - 1 => {
            full_name_without_generics.split_at(index + 1).1.to_string()
        }
        _ => full_name_without_generics.to_string(),
    };
    format!("{}{short_name}", &full_name[..name_start_index])
}

/// Returns indefinite article "a" or "an" that corresponds to the `word`,
/// or an empty string if the indefinite article do not fit to the word.
///
/// Note that the function does not recognize plurals and assumes that the
/// `word` is in singular.
///
/// If an article is returned, it is followed by a space, e.g. "a ".
pub fn a_or_an<S: AsRef<str> + ?Sized>(word: &S) -> &'static str {
    let is_a = in_definite::is_an(word.as_ref());
    match is_a {
        in_definite::Is::An => "an ",
        in_definite::Is::A => "a ",
        in_definite::Is::None => "",
    }
}

/// Returns the ordinal suffix for the given `num`.
/// Convenient for building ordinal numbers like "1st", "2nd", "3rd", "4th", etc.
pub fn ord_num_suffix(num: usize) -> &'static str {
    match num % 100 {
        11..=13 => "th",
        _ => match num % 10 {
            1 => "st",
            2 => "nd", // typos:ignore
            3 => "rd",
            _ => "th",
        },
    }
}

/// Returns `text` with the first character turned into ASCII uppercase.
pub fn ascii_sentence_case(text: &String) -> Cow<String> {
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
pub fn first_line(text: &str, with_ellipses: bool) -> Cow<str> {
    if !text.contains('\n') {
        Cow::Borrowed(text)
    } else {
        let index_of_new_line = text.find('\n').unwrap();
        Cow::Owned(text[..index_of_new_line].to_string() + if with_ellipses { "..." } else { "" })
    }
}

/// Finds strings from an iterable of `possible_values` similar to a given value `v`.
/// Returns a vector of all possible values that exceed a similarity threshold,
/// sorted by similarity (most similar comes first). The returned vector will have
/// at most `max_num_of_suggestions` elements.
///
/// The implementation is taken and adapted from the [Clap project](https://github.com/clap-rs/clap/blob/50f7646cf72dd7d4e76d9284d76bdcdaceb7c049/clap_builder/src/parser/features/suggestions.rs#L11).
pub fn did_you_mean<T, I>(v: &str, possible_values: I, max_num_of_suggestions: usize) -> Vec<String>
where
    T: AsRef<str>,
    I: IntoIterator<Item = T>,
{
    let mut candidates: Vec<_> = possible_values
        .into_iter()
        .map(|pv| (strsim::jaro(v, pv.as_ref()), pv.as_ref().to_owned()))
        // Confidence of 0.7 so that bar -> baz is suggested.
        .filter(|(confidence, _)| *confidence > 0.7)
        .collect();
    candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal));
    candidates
        .into_iter()
        .take(max_num_of_suggestions)
        .map(|(_, pv)| pv)
        .collect()
}

/// Returns a single line "Did you mean" [Hint::help]. E.g.: Did you mean "this" or "that"?
///
/// The input value is taken from the `span` and the help hint is positioned at that `span`.
/// Each suggestion are enclosed in `enclosing`.
pub fn did_you_mean_help<T, I>(
    source_engine: &SourceEngine,
    span: Span,
    possible_values: I,
    max_num_of_suggestions: usize,
    enclosing: Enclosing,
) -> Hint
where
    T: AsRef<str>,
    I: IntoIterator<Item = T>,
{
    let suggestions = &did_you_mean(span.as_str(), possible_values, max_num_of_suggestions);
    if suggestions.is_empty() {
        Hint::none()
    } else {
        Hint::help(
            source_engine,
            span,
            format!(
                "Did you mean {}?",
                sequence_to_str_or(suggestions, enclosing, max_num_of_suggestions)
            ),
        )
    }
}

mod test {
    #[test]
    fn test_short_name() {
        use super::short_name;

        let test = |full_name: &str, expected: &str| {
            let short_name = short_name(full_name);
            assert_eq!(short_name, expected, "Full name: {full_name}.");
        };

        test("SomeType", "SomeType");
        test("&SomeType", "&SomeType");
        test("&&&SomeType", "&&&SomeType");
        test("&mut &&mut SomeType", "&mut &&mut SomeType");
        test("&&&mut &mut SomeType", "&&&mut &mut SomeType");
        test("SomeType<T>", "SomeType");
        test("&SomeType<&T>", "&SomeType");
        test("&&&SomeType<&&&T>", "&&&SomeType");
        test("&mut &&mut SomeType<&mut &&mut T>", "&mut &&mut SomeType");
        test(
            "&&&mut &mut SomeType<&&&mut &mut T>",
            "&&&mut &mut SomeType",
        );
        test("std::ops::Eq", "Eq");
        test("some_lib::Struct<A, B>", "Struct");
        test("&&mut some_lib::Struct<&A, &mut B>", "&&mut Struct");
        test(
            "some_lib::Struct<some::other::lib::A, some::other::lib::B>",
            "Struct",
        );
        test(
            "&&&mut some_lib::Struct<some::other::lib::A, some::other::lib::B>",
            "&&&mut Struct",
        );
        test(
            "some_lib::fn::function<some::other::lib::A<T1, T2>, some::other::lib::B<T3>>",
            "function",
        );
    }
}
