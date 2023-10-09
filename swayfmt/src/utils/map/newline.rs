use anyhow::Result;
use ropey::Rope;
use std::{
    collections::BTreeMap,
    iter::{Enumerate, Peekable},
    path::PathBuf,
    str::Chars,
    sync::Arc,
};
use sway_ast::Module;
use sway_types::SourceEngine;

use crate::{
    constants::NEW_LINE,
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
            .map(|_| NEW_LINE)
            .collect::<String>()
    }
}

type NewlineMap = BTreeMap<ByteSpan, NewlineSequence>;

/// Checks if there is a new line at the current position of the rope
#[inline]
fn is_new_line_in_rope(rope: &Rope, index: usize) -> bool {
    for (p, new_line) in NEW_LINE.chars().enumerate() {
        if rope.get_char(index + p) != Some(new_line) {
            return false;
        }
    }
    true
}

/// Checks if there is a new line (Operating system specific) next in the iter of characters
#[inline]
fn is_new_line_next_in_iter(
    input_iter: &mut Peekable<Enumerate<Chars<'_>>>,
    new_line: &[char],
) -> bool {
    let total = new_line.len();
    for (p, char_new_line) in new_line.iter().enumerate() {
        if input_iter.peek().map(|x| x.1) == Some(*char_new_line) {
            if total != p + 1 {
                input_iter.next();
            }
        } else {
            return false;
        }
    }

    true
}

/// Search for newline sequences in the unformatted code and collect ByteSpan -> NewlineSequence for the input source
fn newline_map_from_src(unformatted_input: &str) -> Result<NewlineMap, FormatterError> {
    let mut newline_map = BTreeMap::new();
    // Iterate over the unformatted source code to find NewlineSequences
    let mut input_iter = unformatted_input.chars().enumerate().peekable();
    let mut current_sequence_length = 0;
    let mut in_sequence = false;
    let mut sequence_start = 0;
    let os_new_line = NEW_LINE.chars().collect::<Vec<_>>();
    while let Some((char_index, char)) = input_iter.next() {
        let next_char = input_iter.peek().map(|input| input.1);
        if (char == '}' || char == ';') && is_new_line_next_in_iter(&mut input_iter, &os_new_line) {
            if !in_sequence {
                sequence_start = char_index + os_new_line.len();
                in_sequence = true;
            }
        } else if os_new_line.ends_with(&[char]) && in_sequence {
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
        if !is_new_line_next_in_iter(&mut input_iter, &os_new_line)
            && current_sequence_length > 0
            && in_sequence
        {
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
    source_engine: &SourceEngine,
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
    let formatted_module = parse_file(source_engine, formatted_input, path)?.value;
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

#[inline]
/// Tiny function that safely calculates the offset where the offset is a i64
/// (it may be negative). If the offset is a negative number and the result of
/// doing the offset would have panic (because usize cannot be negative) the
/// unmodified base would be returned
fn calculate_offset(base: usize, offset: i64) -> usize {
    offset
        .checked_add(base as i64)
        .unwrap_or(base as i64)
        .try_into()
        .unwrap_or(base)
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
            // At its core, the spaces between leaf spans are nothing more than just whitespace characters,
            // and sometimes comments, since they are not considered valid AST nodes. We are interested in
            // these spaces (with comments, if any)
            let whitespaces_with_comments = &unformatted_code
                [previous_unformatted_newline_span.end..unformatted_newline_span.start];

            let mut whitespaces_with_comments_it =
                whitespaces_with_comments.char_indices().peekable();

            let start = previous_unformatted_newline_span.end;
            let mut comment_found = false;

            // Here, we will try to insert newlines that occur before comments.
            while let Some((idx, character)) = whitespaces_with_comments_it.next() {
                if character == '/' {
                    if let Some((_, '/') | (_, '*')) = whitespaces_with_comments_it.peek() {
                        comment_found = true;

                        // Insert newlines that occur before the first comment here
                        if let Some(newline_sequence) = first_newline_sequence_in_span(
                            &ByteSpan {
                                start,
                                end: start + idx,
                            },
                            &newline_map,
                        ) {
                            offset += insert_after_span(
                                calculate_offset(previous_formatted_newline_span.end, offset),
                                newline_sequence,
                                formatted_code,
                                newline_threshold,
                            )?;
                            break;
                        }
                    }
                }

                // If there are no comments found in the sequence of whitespaces, there is no point
                // in trying to find newline sequences from the back. So we simply take the entire
                // sequence, insert newlines at the start and we're done with this iteration of the for loop.
                if idx == whitespaces_with_comments.len() - 1 && !comment_found {
                    if let Some(newline_sequence) = first_newline_sequence_in_span(
                        &ByteSpan {
                            start,
                            end: unformatted_newline_span.start,
                        },
                        &newline_map,
                    ) {
                        offset += insert_after_span(
                            calculate_offset(previous_formatted_newline_span.end, offset),
                            newline_sequence,
                            formatted_code,
                            newline_threshold,
                        )?;
                    }
                }
            }

            // If we found some comment(s), we are also interested in inserting
            // newline sequences that happen after the last comment.
            //
            // This can be a single comment or multiple comments.
            if comment_found {
                let mut whitespaces_with_comments_rev_it =
                    whitespaces_with_comments.char_indices().rev().peekable();
                let mut end_of_last_comment = whitespaces_with_comments.len();

                // Find point of insertion of newline sequences
                for (idx, character) in whitespaces_with_comments_rev_it.by_ref() {
                    if !character.is_whitespace() {
                        end_of_last_comment = idx + 1;
                        break;
                    }
                }

                while let Some((_, character)) = whitespaces_with_comments_rev_it.next() {
                    if character == '/' {
                        // Comments either start with '//' or end with '*/'
                        if let Some((_, '/') | (_, '*')) = whitespaces_with_comments_rev_it.peek() {
                            if let Some(newline_sequence) = first_newline_sequence_in_span(
                                &ByteSpan {
                                    start: start + end_of_last_comment,
                                    end: unformatted_newline_span.start,
                                },
                                &newline_map,
                            ) {
                                offset += insert_after_span(
                                    calculate_offset(
                                        previous_formatted_newline_span.end + end_of_last_comment,
                                        offset,
                                    ),
                                    newline_sequence,
                                    formatted_code,
                                    newline_threshold,
                                )?;
                            }
                            break;
                        }
                    }
                }
            }
        }
        previous_unformatted_newline_span = unformatted_newline_span;
        previous_formatted_newline_span = formatted_newline_span;
    }
    Ok(())
}

fn format_newline_sequence(newline_sequence: &NewlineSequence, threshold: usize) -> String {
    if newline_sequence.sequence_length > threshold {
        (0..threshold).map(|_| NEW_LINE).collect::<String>()
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
) -> Result<i64, FormatterError> {
    let sequence_string = format_newline_sequence(&newline_sequence, threshold);
    let mut len = sequence_string.len() as i64;
    let mut src_rope = Rope::from_str(formatted_code);

    // Remove the previous sequence_length, that will be replaced in the next statement
    let mut remove_until = at;
    for i in at..at + newline_sequence.sequence_length {
        if !is_new_line_in_rope(&src_rope, i) {
            break;
        }
        remove_until = i;
    }
    if remove_until > at {
        src_rope
            .try_remove(at..remove_until)
            .map_err(|_| FormatterError::NewlineSequenceError)?;
        len -= (remove_until - at) as i64;
    }

    src_rope
        .try_insert(at, &sequence_string)
        .map_err(|_| FormatterError::NewlineSequenceError)?;

    formatted_code.clear();
    formatted_code.push_str(&src_rope.to_string());
    Ok(len)
}

/// Returns the first newline sequence contained in a span.
/// This is inclusive at the start, and exclusive at the end, i.e.
/// the bounds are [span.start, span.end).
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
