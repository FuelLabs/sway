use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

use sway_ast::Module;

use crate::{fmt::FormattedCode, FormatterError};

use super::byte_span::ByteSpan;

/// Represents a series of consecutive newlines
#[derive(Debug)]
struct NewlineSequence {
    sequence_length: usize,
}

type NewlineMap = BTreeMap<ByteSpan, NewlineSequence>;

/// Search for newline sequences in the unformatted code and collect ByteSpan -> NewlineSequence for the input source
fn newline_map_from_src(unformatted_input: Arc<str>) -> Result<NewlineMap, FormatterError> {
    let mut newline_map = BTreeMap::new();
    // Iterate over the unformatted source code to find NewlineSequences
    let mut input_iter = unformatted_input.chars().enumerate().peekable();
    let mut current_sequence_length = 0;
    let mut in_sequence = false;
    let mut sequence_start = 0;
    while let Some((char_index, char)) = input_iter.next() {
        if char == '\n' {
            if !in_sequence {
                sequence_start = char_index;
                in_sequence = true;
            }
            current_sequence_length += 1;
        } else if Some('\n') != input_iter.peek().map(|input| input.1) {
            if current_sequence_length > 0 {
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
    }
    Ok(newline_map)
}

/// Handle newlines by first creating a NewlineMap which is used for fast searching extra newlines.
/// Traverses items for finding a newline sequence in unformatted input and placing it in correct place in formatted output.
pub fn handle_newlines(
    _unformatted_input: Arc<str>,
    _unformatted_module: &Module,
    _formatted_input: Arc<str>,
    _path: Option<Arc<PathBuf>>,
    _formatted_code: &mut FormattedCode,
) {
    // Collect Span -> NewlineSequence mapping from unformatted input
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::newline_map_from_src;
    #[test]
    fn test_newline_sequences() {
        let raw_src = r#"script;

fn main() {
    let number: u64 = 10;

    let number2: u64 = 20;


    let number3: u64 = 30;
}"#;

        let newline_map = newline_map_from_src(Arc::from(raw_src)).unwrap();
        let newline_sequence_lengths = Vec::from_iter(
            newline_map
                .iter()
                .map(|map_item| map_item.1.sequence_length),
        );
        let correct_newline_sequence_lengths = vec![2, 1, 2, 3, 1];

        assert_eq!(newline_sequence_lengths, correct_newline_sequence_lengths);
    }
}
