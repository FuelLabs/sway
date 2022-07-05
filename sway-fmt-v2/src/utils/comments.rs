use anyhow::Result;
use std::{cmp::Ordering, collections::BTreeMap, sync::Arc};
use sway_parse::token::{lex_commented, Comment, CommentedTokenTree};

/// Represents a span for the comments in a spesific file
/// A stripped down version of sway-types::src::Span
#[derive(PartialEq, Eq, Debug)]
pub struct CommentSpan {
    // The byte position in the string of the start of the span.
    start: usize,
    // The byte position in the string of the end of the span.
    end: usize,
}

impl Ord for CommentSpan {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start.cmp(&other.start)
    }
}

impl PartialOrd for CommentSpan {
    fn partial_cmp(&self, other: &CommentSpan) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub type CommentMap = BTreeMap<CommentSpan, Comment>;

/// Get the CommentedTokenStream and collect the spans -> Comment mapping for the input source
/// code.
pub fn construct_comment_map(input: Arc<str>) -> Result<CommentMap> {
    let mut comment_map = BTreeMap::new();

    // pass the input through lexer
    let commented_token_stream = lex_commented(&input, 0, input.len(), None)?;
    let tts = commented_token_stream.token_trees().iter();

    for comment in tts {
        if let CommentedTokenTree::Comment(comment) = comment {
            let comment_span = CommentSpan {
                start: comment.span.start(),
                end: comment.span.end(),
            };
            comment_map.insert(comment_span, comment.clone());
        }
        // TODO: implement CommentedTokenTree::Tree case for getting the comments inside code
        // blocks
    }
    Ok(comment_map)
}

#[cfg(test)]
mod tests {
    use super::{construct_comment_map, CommentSpan};
    use std::{ops::Bound::Included, sync::Arc};

    #[test]
    fn test_comment_span_map() {
        let input = r#"
        // Single-line comment.
        struct Foo {
            /* multi-
             * line-
             * comment */
            bar: i32,
        }
        "#;
        let map = construct_comment_map(Arc::from(input)).unwrap();
        assert!(map.len() != 0);
        let range_start_span = CommentSpan { start: 0, end: 32 };
        let range_end_span = CommentSpan { start: 33, end: 34 };
        let found_comment = map
            .range((Included(range_start_span), Included(range_end_span)))
            .last()
            .unwrap();
        assert_eq!(found_comment.1.span.as_str(), "// Single-line comment.");
    }
}
