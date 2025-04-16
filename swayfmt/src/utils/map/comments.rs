use super::byte_span;
use crate::{formatter::FormatterError, parse::lex, utils::map::byte_span::ByteSpan};
use std::{
    collections::BTreeMap,
    ops::{
        Bound::{Excluded, Included},
        Deref, DerefMut, Range,
    },
};
use sway_ast::token::{Comment, CommentedTokenTree, CommentedTree};
use sway_types::span::Source;

#[derive(Clone, Default, Debug)]
pub struct CommentMap(pub BTreeMap<ByteSpan, Comment>);

impl Deref for CommentMap {
    type Target = BTreeMap<ByteSpan, Comment>;

    fn deref(&self) -> &BTreeMap<ByteSpan, Comment> {
        &self.0
    }
}

impl DerefMut for CommentMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl CommentMap {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Get the CommentedTokenStream and collect the spans -> Comment mapping for the input source
    /// code.
    pub fn from_src(input: Source) -> Result<Self, FormatterError> {
        // Pass the input through the lexer.
        let tts = lex(input)?;
        let tts = tts.token_trees().iter();

        let mut comment_map = CommentMap::new();
        for comment in tts {
            comment_map.collect_comments_from_token_stream(comment);
        }
        Ok(comment_map)
    }

    /// Given a range, return an iterator to comments contained within the range.
    pub fn comments_between<'a>(
        &'a self,
        range: &'a Range<usize>,
    ) -> impl Iterator<Item = &'a Comment> {
        self.iter().filter_map(|(bs, c)| {
            if bs.contained_within(range) {
                Some(c)
            } else {
                None
            }
        })
    }

    /// Collects `Comment`s from the token stream and insert it with its span to the `CommentMap`.
    /// Handles both the standalone and in-block comments.
    fn collect_comments_from_token_stream(&mut self, commented_token_tree: &CommentedTokenTree) {
        match commented_token_tree {
            CommentedTokenTree::Comment(comment) => {
                let comment_span = ByteSpan {
                    start: comment.span.start(),
                    end: comment.span.end(),
                };
                self.insert(comment_span, comment.clone());
            }
            CommentedTokenTree::Tree(CommentedTree::Group(group)) => {
                for item in group.token_stream.token_trees().iter() {
                    self.collect_comments_from_token_stream(item);
                }
            }
            _ => {}
        }
    }
}

trait CommentRange {
    /// Get comments in between given ByteSpans. This is wrapper around BtreeMap::range with a custom logic for beginning of the file.
    fn comments_in_range(&self, from: &ByteSpan, to: &ByteSpan) -> Vec<(ByteSpan, Comment)>;
}

impl CommentRange for CommentMap {
    fn comments_in_range(&self, from: &ByteSpan, to: &ByteSpan) -> Vec<(ByteSpan, Comment)> {
        // While searching for comments with given range, comment handler needs to check if the beginning of the range is actually the beginning of the file.
        // If that is the case we need to collect all the comments until the provided `to` ByteSpan. BtreeMap::range((Inclusive(from), Excluded(to))) won't be able to find comments
        // since both beginning of the file and first byte span have their start = 0. If we are looking from STARTING_BYTE_SPAN to `to`, we need to collect all until `to` byte span.
        if from == &byte_span::STARTING_BYTE_SPAN {
            self.range(..to)
                .map(|(byte_span, comment)| (byte_span.clone(), comment.clone()))
                .collect()
        } else {
            self.range((Included(from), Excluded(to)))
                .map(|(byte_span, comment)| (byte_span.clone(), comment.clone()))
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ByteSpan;
    use crate::utils::map::comments::{CommentMap, CommentRange};
    use std::ops::Bound::Included;

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
        let map = CommentMap::from_src(input.into()).unwrap();
        assert!(!map.is_empty());
        let range_start_span = ByteSpan { start: 0, end: 32 };
        let range_end_span = ByteSpan { start: 33, end: 34 };
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
        let map = CommentMap::from_src(input.into()).unwrap();
        assert!(!map.is_empty());
        let range_start_span = ByteSpan { start: 40, end: 54 };
        let range_end_span = ByteSpan {
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
        let map = CommentMap::from_src(input.into()).unwrap();
        assert!(!map.is_empty());
        let range_start_span = ByteSpan {
            start: 110,
            end: 116,
        };
        let range_end_span = ByteSpan {
            start: 200,
            end: 201,
        };
        let found_comment = map
            .range((Included(range_start_span), Included(range_end_span)))
            .last()
            .unwrap();
        assert_eq!(
            found_comment.1.span.as_str(),
            "/* multi-\n             * line-\n             * comment */"
        );
    }
    #[test]
    fn test_comment_map_range_from_start() {
        let range_start_span = ByteSpan { start: 0, end: 0 };
        let range_end_span = ByteSpan { start: 8, end: 16 };
        let input = r#"// test
contract;"#;
        let map = CommentMap::from_src(input.into()).unwrap();
        assert!(!map.is_empty());
        let found_comments = map.comments_in_range(&range_start_span, &range_end_span);
        assert_eq!(found_comments[0].1.span.as_str(), "// test");
    }
}
