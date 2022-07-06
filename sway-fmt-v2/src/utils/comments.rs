use anyhow::Result;
use std::{cmp::Ordering, collections::BTreeMap, sync::Arc};
use sway_parse::token::{lex_commented, Comment, CommentedTokenTree, CommentedTree};

/// Represents a span for the comments in a spesific file
/// A stripped down version of sway-types::src::Span
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct CommentSpan {
    // The byte position in the string of the start of the span.
    start: usize,
    // The byte position in the string of the end of the span.
    end: usize,
}

impl Ord for CommentSpan {
    fn cmp(&self, other: &Self) -> Ordering {
        // If the starting position is the same encapsulatig span (i.e, wider one) should come
        // first
        match self.start.cmp(&other.start) {
            Ordering::Equal => self.end.cmp(&other.end),
            ord => ord,
        }
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
        get_comment_from_token_stream(comment, &mut comment_map);
    }
    Ok(comment_map)
}

/// Get `Comment` from the token stream and insert it with its span to the `CommentMap`.
/// Handles both the standalone and in-block comments.
fn get_comment_from_token_stream(
    commented_token_tree: &CommentedTokenTree,
    comment_map: &mut CommentMap,
) {
    match commented_token_tree {
        CommentedTokenTree::Comment(comment) => {
            let comment_span = CommentSpan {
                start: comment.span.start(),
                end: comment.span.end(),
            };
            comment_map.insert(comment_span, comment.clone());
        }
        CommentedTokenTree::Tree(CommentedTree::Group(group)) => {
            for item in group.token_stream.token_trees().iter() {
                get_comment_from_token_stream(item, comment_map);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{construct_comment_map, CommentSpan};
    use std::{ops::Bound::Included, sync::Arc};

    #[test]
    fn test_comment_span_ordering() {
        let first_span = CommentSpan { start: 2, end: 4 };
        let second_span = CommentSpan { start: 2, end: 6 };
        let third_span = CommentSpan { start: 4, end: 7 };

        let mut vec = vec![second_span.clone(), third_span.clone(), first_span.clone()];
        vec.sort();

        assert_eq!(vec[0], first_span);
        assert_eq!(vec[1], second_span);
        assert_eq!(vec[2], third_span);
    }

    #[test]
    fn test_comment_span_map_standalone_comment() {
        let input = r#"
        // Single-line comment.
        let var = 256; // This is a comment.
        struct Foo {
            /* multi-
             * line-
             * comment */
            bar: i32,
        }
        "#;
        let map = construct_comment_map(Arc::from(input)).unwrap();
        assert!(!map.is_empty());
        let range_start_span = CommentSpan { start: 0, end: 32 };
        let range_end_span = CommentSpan { start: 33, end: 34 };
        let found_comment = map
            .range((Included(range_start_span), Included(range_end_span)))
            .last()
            .unwrap();
        assert_eq!(found_comment.1.span.as_str(), "// Single-line comment.");
    }
    #[test]
    fn test_comment_span_map_standalone_next_to_item() {
        let input = r#"
        // Single-line comment.
        let var = 256; // This is a comment.
        struct Foo {
            /* multi-
             * line-
             * comment */
            bar: i32,
        }
        "#;
        let map = construct_comment_map(Arc::from(input)).unwrap();
        assert!(!map.is_empty());
        let range_start_span = CommentSpan { start: 40, end: 54 };
        let range_end_span = CommentSpan {
            start: 100,
            end: 115,
        };
        let found_comment = map
            .range((Included(range_start_span), Included(range_end_span)))
            .last()
            .unwrap();
        assert_eq!(found_comment.1.span.as_str(), "// This is a comment.");
    }
    #[test]
    fn test_comment_span_map_standalone_inside_block() {
        let input = r#"
        // Single-line comment.
        let var = 256; // This is a comment.
        struct Foo {
            /* multi-
             * line-
             * comment */
            bar: i32,
        }
        "#;
        let map = construct_comment_map(Arc::from(input)).unwrap();
        assert!(!map.is_empty());
        let range_start_span = CommentSpan {
            start: 110,
            end: 116,
        };
        let range_end_span = CommentSpan {
            start: 200,
            end: 201,
        };
        let found_comment = map
            .range((Included(range_start_span), Included(range_end_span)))
            .last()
            .unwrap();
        assert_eq!(
            found_comment.1.span.as_str(),
            "/* multi-\n             * line-\n             * comment *"
        );
    }
}
