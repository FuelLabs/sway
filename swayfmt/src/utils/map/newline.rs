use anyhow::Result;
use ropey::Rope;
use std::{collections::BTreeMap, fmt::Write, path::PathBuf, sync::Arc};
use sway_ast::Module;

use crate::{
    formatter::{FormattedCode, Formatter},
    parse::parse_file,
    utils::map::byte_span::{ByteSpan, LeafSpans},
    FormatterError,
};

/// Represents a series of consecutive newlines
#[derive(Debug, Clone, PartialEq)]
struct NewlineSequence {
    sequence_length: usize,
}

impl ToString for NewlineSequence {
    fn to_string(&self) -> String {
        (0..self.sequence_length - 1)
            .map(|_| "\n")
            .collect::<String>()
    }
}

type NewlineMap = BTreeMap<ByteSpan, NewlineSequence>;

/// Search for newline sequences in the unformatted code and collect ByteSpan -> NewlineSequence for the input source
fn newline_map_from_src(unformatted_input: &str) -> Result<NewlineMap, FormatterError> {
    let mut newline_map = BTreeMap::new();
    // Iterate over the unformatted source code to find NewlineSequences
    let mut input_iter = unformatted_input.chars().enumerate().peekable();
    let mut current_sequence_length = 0;
    let mut in_sequence = false;
    let mut sequence_start = 0;
    while let Some((char_index, char)) = input_iter.next() {
        let next_char = input_iter.peek().map(|input| input.1);
        if (char == '}' || char == ';') && next_char == Some('\n') {
            if !in_sequence {
                sequence_start = char_index + 1;
                in_sequence = true;
            }
        } else if char == '\n' && in_sequence {
            current_sequence_length += 1;
        }
        if (Some('}') == next_char || Some('(') == next_char) && in_sequence {
            // If we are in a sequence and find `}`, abort the sequence
            current_sequence_length = 0;
            in_sequence = false;
        }
        if next_char == Some(' ') || next_char == Some('\t') {
            continue;
        }
        if Some('\n') != next_char && current_sequence_length > 0 && in_sequence {
            // Next char is not a newline so this is the end of the sequence
            let byte_span = ByteSpan {
                start: sequence_start,
                end: char_index,
            };
            let newline_sequence = NewlineSequence {
                sequence_length: current_sequence_length,
            };
            newline_map.insert(byte_span, newline_sequence);
            current_sequence_length = 0;
            in_sequence = false;
        }
    }
    Ok(newline_map)
}

/// Handle newlines by first creating a NewlineMap which is used for fast searching extra newlines.
/// Traverses items for finding a newline sequence in unformatted input and placing it in correct place in formatted output.
pub fn handle_newlines(
    unformatted_input: Arc<str>,
    unformatted_module: &Module,
    formatted_input: Arc<str>,
    path: Option<Arc<PathBuf>>,
    formatted_code: &mut FormattedCode,
    formatter: &Formatter,
) -> Result<(), FormatterError> {
    // Get newline threshold from config
    let newline_threshold = formatter.config.whitespace.newline_threshold;
    // Collect ByteSpan -> NewlineSequence mapping from unformatted input.
    //
    // We remove the extra whitespace the beginning of a file before creating a map of newlines.
    // This is to avoid conflicts with logic that determine comment formatting, and ensure
    // formatting the code a second time will still produce the same result.
    let newline_map = newline_map_from_src(&unformatted_input)?;
    // After the formatting existing items should be the same (type of the item) but their spans will be changed since we applied formatting to them.
    let formatted_module = parse_file(formatted_input, path)?.value;
    // Actually find & insert the newline sequences
    add_newlines(
        newline_map,
        unformatted_module,
        &formatted_module,
        formatted_code,
        unformatted_input,
        newline_threshold,
    )?;
    Ok(())
}
/// Adds the newlines from newline_map to correct places in the formatted code. This requires us
/// both the unformatted and formatted code's modules as they will have different spans for their
/// visitable positions. While traversing the unformatted module, `add_newlines` searches for newline sequences. If there is a newline sequence found
/// it places the sequence to the correct place at formatted_code.
///
/// This requires both the unformatted_code itself and the parsed version of it, because
/// unformatted_code is used for context lookups and unformatted_module is required for actual
/// traversal.
fn add_newlines(
    newline_map: NewlineMap,
    unformatted_module: &Module,
    formatted_module: &Module,
    formatted_code: &mut FormattedCode,
    unformatted_code: Arc<str>,
    newline_threshold: usize,
) -> Result<(), FormatterError> {
    let mut unformatted_newline_spans = unformatted_module.leaf_spans();
    let mut formatted_newline_spans = formatted_module.leaf_spans();
    // Adding end of file to both spans so that last newline sequence(s) after an item would also be
    // found & included
    unformatted_newline_spans.push(ByteSpan {
        start: unformatted_code.len(),
        end: unformatted_code.len(),
    });
    formatted_newline_spans.push(ByteSpan {
        start: formatted_code.len(),
        end: formatted_code.len(),
    });
    // Since we are adding newline sequences into the formatted code, in the next iteration the spans we find for the formatted code needs to be offsetted
    // as the total length of newline sequences we added in previous iterations.
    let mut offset = 0;
    // We will definetly have a span in the collected span since for a source code to be parsed there should be some tokens present.
    let mut previous_unformatted_newline_span = unformatted_newline_spans
        .first()
        .ok_or(FormatterError::NewlineSequenceError)?;
    let mut previous_formatted_newline_span = formatted_newline_spans
        .first()
        .ok_or(FormatterError::NewlineSequenceError)?;
    for (unformatted_newline_span, formatted_newline_span) in unformatted_newline_spans
        .iter()
        .skip(1)
        .zip(formatted_newline_spans.iter().skip(1))
    {
        if previous_unformatted_newline_span.end < unformatted_newline_span.start {
            let snippet = &unformatted_code
                [previous_unformatted_newline_span.end..unformatted_newline_span.start];

            let start = previous_unformatted_newline_span.end;

            // Here, we will try to insert newlines that occur before and after comments.
            // The reason we do this is because comments aren't a part of the AST, and so they aren't
            // collected as leaf spans - they simply exist between whitespaces.
            if let Some(start_first_comment) = snippet.find("//") {
                // Insert newlines that occur before the first comment here
                if let Some(newline_sequence) = first_newline_sequence_in_span(
                    &ByteSpan {
                        start,
                        end: start + start_first_comment,
                    },
                    &newline_map,
                ) {
                    let at = previous_formatted_newline_span.end + offset;
                    offset +=
                        insert_after_span(at, newline_sequence, formatted_code, newline_threshold)?;
                }

                // Insert newlines that occur after the last comment here
                if let Some(start_last_comment) = snippet.rfind("//") {
                    if start_first_comment != start_last_comment {
                        if let Some(end_last_comment) = snippet[start_last_comment..].find('\n') {
                            if let Some(newline_sequence) = first_newline_sequence_in_span(
                                &ByteSpan {
                                    start: start + start_last_comment + end_last_comment,
                                    end: unformatted_newline_span.start,
                                },
                                &newline_map,
                            ) {
                                let at = previous_formatted_newline_span.end
                                    + offset
                                    + start_last_comment
                                    + end_last_comment;
                                offset += insert_after_span(
                                    at,
                                    newline_sequence,
                                    formatted_code,
                                    newline_threshold,
                                )?;
                            }
                        }
                    }
                }
            } else if let Some(newline_sequence) = first_newline_sequence_in_span(
                &ByteSpan {
                    start,
                    end: unformatted_newline_span.start,
                },
                &newline_map,
            ) {
                let at = previous_formatted_newline_span.end + offset;
                offset +=
                    insert_after_span(at, newline_sequence, formatted_code, newline_threshold)?;
            }
        }
        previous_unformatted_newline_span = unformatted_newline_span;
        previous_formatted_newline_span = formatted_newline_span;
    }
    Ok(())
}

fn format_newline_sequence(newline_sequence: &NewlineSequence, threshold: usize) -> String {
    if newline_sequence.sequence_length > threshold {
        (0..threshold).map(|_| "\n").collect::<String>()
    } else {
        newline_sequence.to_string()
    }
}

/// Inserts a `NewlineSequence` at position `at` and returns the length of `NewlineSequence` inserted.
/// The return value is used to calculate the new `at` in a later point.
fn insert_after_span(
    at: usize,
    newline_sequence: NewlineSequence,
    formatted_code: &mut FormattedCode,
    threshold: usize,
) -> Result<usize, FormatterError> {
    let mut sequence_string = String::new();
    write!(
        sequence_string,
        "{}",
        format_newline_sequence(&newline_sequence, threshold)
    )?;
    let mut src_rope = Rope::from_str(formatted_code);
    src_rope.insert(at, &sequence_string);
    formatted_code.clear();
    formatted_code.push_str(&src_rope.to_string());
    Ok(sequence_string.len())
}

/// Returns a newline sequence contained in a span.
fn first_newline_sequence_in_span(
    span: &ByteSpan,
    newline_map: &NewlineMap,
) -> Option<NewlineSequence> {
    for (range, sequence) in newline_map.iter() {
        if span.start <= range.start && range.end < span.end {
            return Some(sequence.clone());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::utils::map::{byte_span::ByteSpan, newline::first_newline_sequence_in_span};

    use super::{newline_map_from_src, NewlineMap, NewlineSequence};

    #[test]
    fn test_newline_map() {
        let raw_src = r#"script;

fn main() {
    let number: u64 = 10;

    let number2: u64 = 20;


    let number3: u64 = 30;



}"#;

        let newline_map = newline_map_from_src(raw_src.trim_start()).unwrap();
        let newline_sequence_lengths = Vec::from_iter(
            newline_map
                .iter()
                .map(|map_item| map_item.1.sequence_length),
        );
        let correct_newline_sequence_lengths = vec![2, 2, 3];

        assert_eq!(newline_sequence_lengths, correct_newline_sequence_lengths);
    }

    #[test]
    fn test_newline_map_with_whitespaces() {
        let raw_src = r#"script;
        fuel_coin.mint        {
            gas:             default_gas
        }

        (11);"#;

        let newline_map = newline_map_from_src(raw_src.trim_start()).unwrap();
        let newline_sequence_lengths = Vec::from_iter(
            newline_map
                .iter()
                .map(|map_item| map_item.1.sequence_length),
        );
        let correct_newline_sequence_lengths = vec![1];

        assert_eq!(newline_sequence_lengths, correct_newline_sequence_lengths);
    }

    #[test]
    fn test_newline_range_simple() {
        let mut newline_map = NewlineMap::new();
        let newline_sequence = NewlineSequence { sequence_length: 2 };

        newline_map.insert(ByteSpan { start: 9, end: 10 }, newline_sequence.clone());
        assert_eq!(
            newline_sequence,
            first_newline_sequence_in_span(&ByteSpan { start: 8, end: 11 }, &newline_map).unwrap()
        );
        assert_eq!(
            newline_sequence,
            first_newline_sequence_in_span(&ByteSpan { start: 9, end: 11 }, &newline_map).unwrap()
        );
        assert!(
            first_newline_sequence_in_span(&ByteSpan { start: 9, end: 10 }, &newline_map).is_none()
        );
        assert!(
            first_newline_sequence_in_span(&ByteSpan { start: 9, end: 9 }, &newline_map).is_none()
        );
        assert!(
            first_newline_sequence_in_span(&ByteSpan { start: 8, end: 8 }, &newline_map).is_none()
        );
    }
}
