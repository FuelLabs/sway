use crate::{
    formatter::{FormattedCode, FormatterError},
    utils::map::byte_span::{ByteSpan, LeafSpans},
};
use ropey::Rope;
use std::{
    collections::BTreeMap,
    fmt::Write,
    ops::Bound::{Excluded, Included},
    path::PathBuf,
    sync::Arc,
};
use sway_ast::{
    token::{Comment, CommentedTokenTree, CommentedTree},
    Module,
};
use sway_parse::lex_commented;
use sway_types::Spanned;

pub type CommentMap = BTreeMap<ByteSpan, Comment>;

/// Get the CommentedTokenStream and collect the spans -> Comment mapping for the input source
/// code.
pub fn comment_map_from_src(input: Arc<str>) -> Result<CommentMap, FormatterError> {
    let mut comment_map = BTreeMap::new();

    // pass the input through lexer
    let commented_token_stream = lex_commented(&input, 0, input.len(), None)?;
    let tts = commented_token_stream.token_trees().iter();

    for comment in tts {
        collect_comments_from_token_stream(comment, &mut comment_map);
    }
    Ok(comment_map)
}

/// Collects `Comment`s from the token stream and insert it with its span to the `CommentMap`.
/// Handles both the standalone and in-block comments.
fn collect_comments_from_token_stream(
    commented_token_tree: &CommentedTokenTree,
    comment_map: &mut CommentMap,
) {
    match commented_token_tree {
        CommentedTokenTree::Comment(comment) => {
            let comment_span = ByteSpan {
                start: comment.span.start(),
                end: comment.span.end(),
            };
            comment_map.insert(comment_span, comment.clone());
        }
        CommentedTokenTree::Tree(CommentedTree::Group(group)) => {
            for item in group.token_stream.token_trees().iter() {
                collect_comments_from_token_stream(item, comment_map);
            }
        }
        _ => {}
    }
}

/// Handles comments by first creating the CommentMap which is used for fast seaching comments.
/// Traverses items for finding a comment in unformatted input and placing it in correct place in formatted output.
pub fn handle_comments(
    unformatted_input: Arc<str>,
    unformatted_module: &Module,
    formatted_input: Arc<str>,
    path: Option<Arc<PathBuf>>,
    formatted_code: &mut FormattedCode,
) -> Result<(), FormatterError> {
    // Collect Span -> Comment mapping from unformatted input.
    let comment_map = comment_map_from_src(unformatted_input.clone())?;

    // After the formatting existing items should be the same (type of the item) but their spans will be changed since we applied formatting to them.
    let formatted_module = sway_parse::parse_file_standalone(formatted_input, path)?;

    // Actually find & insert the comments.
    add_comments(
        comment_map,
        unformatted_module,
        &formatted_module,
        formatted_code,
        unformatted_input,
    )
}

/// Adds the comments from comment_map to correct places in the formatted code. This requires us
/// both the unformatted and formatted code's modules as they will have different spans for their
/// visitable positions. While traversing the unformatted module, `add_comments` searches for comments. If there is a comment found
/// places the comment to the correct place at formatted_code.
///
/// This requires both the unformatted_code itself and the parsed version of it, because
/// unformatted_code is used for context lookups and unformatted_module is required for actual
/// traversal. When `add_comments` is called we have already parsed the unformatted_code so there is no need
/// to parse it again.
fn add_comments(
    comment_map: CommentMap,
    unformatted_module: &Module,
    formatted_module: &Module,
    formatted_code: &mut FormattedCode,
    unformatted_code: Arc<str>,
) -> Result<(), FormatterError> {
    let mut unformatted_comment_spans = unformatted_module.leaf_spans();
    let mut formatted_comment_spans = formatted_module.leaf_spans();
    // Adding end of file to both spans so that the last comment(s) after an item would also be
    // found & included
    unformatted_comment_spans.push(ByteSpan {
        start: unformatted_code.len(),
        end: unformatted_code.len(),
    });
    formatted_comment_spans.push(ByteSpan {
        start: formatted_code.len(),
        end: formatted_code.len(),
    });

    // Since we are adding comments into formatted code, in the next iteration the spans we find for the formatted code needs to be offsetted
    // as the total length of comments we added in previous iterations.
    let mut offset = 0;

    // We will definetly have a span in the collected span since for a source code to be parsed there should be some tokens present.
    let mut previous_unformatted_comment_span = unformatted_comment_spans
        .first()
        .ok_or(FormatterError::CommentError)?;
    let mut previous_formatted_comment_span = formatted_comment_spans
        .first()
        .ok_or(FormatterError::CommentError)?;
    for (unformatted_comment_span, formatted_comment_span) in unformatted_comment_spans
        .iter()
        .skip(1)
        .zip(formatted_comment_spans.iter().skip(1))
    {
        let comments_found = get_comments_between_spans(
            previous_unformatted_comment_span,
            unformatted_comment_span,
            &comment_map,
            &unformatted_code,
        );
        if !comments_found.is_empty() {
            offset += insert_after_span(
                previous_formatted_comment_span,
                comments_found,
                offset,
                formatted_code,
            )?;
        }
        previous_unformatted_comment_span = unformatted_comment_span;
        previous_formatted_comment_span = formatted_comment_span;
    }
    Ok(())
}

// A CommentWithContext is the Comment and the offset before it. The offset can be between the (from) item we searched for this comment or from the last comment inside range
type CommentWithContext = (Comment, String);

/// Returns a list of comments between given spans. For each comment returns the Context
/// Context of a comment is basically the offset (the characters between the last item/comment) to the current comment
fn get_comments_between_spans(
    from: &ByteSpan,
    to: &ByteSpan,
    comment_map: &CommentMap,
    unformatted_code: &Arc<str>,
) -> Vec<CommentWithContext> {
    let mut comments_with_context: Vec<CommentWithContext> = Vec::new();
    for (index, (comment_span, comment)) in comment_map
        .range((Included(from), Excluded(to)))
        .enumerate()
    {
        let starting_position_for_context = if index == 0 {
            // This is the first comment in the current range the context should be collected between from's end and comment's beginning
            from.end
        } else {
            // There is a comment before this one, so we should get the context starting from the last comment's end to the beginning of the current comment
            comments_with_context[index - 1].0.span.end()
        };
        comments_with_context.push((
            comment.clone(),
            unformatted_code[starting_position_for_context..comment_span.start].to_string(),
        ));
    }
    comments_with_context
}

/// Inserts after given span and returns the offset. While inserting comments this also inserts Context of the comments so that the alignment whitespaces/newlines are intact
fn insert_after_span(
    from: &ByteSpan,
    comments_to_insert: Vec<CommentWithContext>,
    offset: usize,
    formatted_code: &mut FormattedCode,
) -> Result<usize, FormatterError> {
    let iter = comments_to_insert.iter();
    let mut offset = offset;
    let mut comment_str = String::new();
    for comment_with_context in iter {
        let (comment_value, comment_context) = comment_with_context;
        write!(
            comment_str,
            "{}{}",
            comment_context,
            &format_comment(comment_value)
        )?;
    }
    let mut src_rope = Rope::from_str(formatted_code);
    // If the position we are going to be inserting from + 1 is a \n we are moving that \n after
    // this comment so if that is the case we are inserting after the \n
    if formatted_code.chars().nth(from.end + offset + 1) == Some('\n') {
        offset += 1;
    }
    src_rope.insert(from.end + offset, &comment_str);
    formatted_code.clear();
    formatted_code.push_str(&src_rope.to_string());
    Ok(comment_str.len())
}

/// Applies formatting to the comment.
/// Currently does not apply any formatting and directly returns the raw comment str
fn format_comment(comment: &Comment) -> String {
    comment.span().str()
}

#[cfg(test)]
mod tests {
    use super::{comment_map_from_src, ByteSpan};
    use std::{ops::Bound::Included, sync::Arc};

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
        let map = comment_map_from_src(Arc::from(input)).unwrap();
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
        let map = comment_map_from_src(Arc::from(input)).unwrap();
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
        let map = comment_map_from_src(Arc::from(input)).unwrap();
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
}
