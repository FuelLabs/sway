use crate::{
    formatter::{FormattedCode, FormatterError},
    parse::{lex, parse_file, parse_snippet},
    utils::map::byte_span::{ByteSpan, LeafSpans},
    Format, Formatter,
};
use ropey::Rope;
use std::{
    collections::BTreeMap,
    fmt::Write,
    ops::{
        Bound::{Excluded, Included},
        Deref, DerefMut, Range,
    },
    path::PathBuf,
    sync::Arc,
};
use sway_ast::{
    token::{Comment, CommentKind, CommentedTokenTree, CommentedTree},
    Module,
};
use sway_types::{Span, Spanned};

use super::byte_span;

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
    pub fn from_src(input: Arc<str>) -> Result<Self, FormatterError> {
        // Pass the input through the lexer.
        let tts = lex(&input)?;
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

/// Handles comments by first creating the CommentMap which is used for fast seaching comments.
/// Traverses items for finding a comment in unformatted input and placing it in correct place in formatted output.
pub fn handle_comments(
    unformatted_input: Arc<str>,
    unformatted_module: &Module,
    formatted_input: Arc<str>,
    path: Option<Arc<PathBuf>>,
    formatted_code: &mut FormattedCode,
    comment_map: &mut CommentMap,
) -> Result<(), FormatterError> {
    // After the formatting existing items should be the same (type of the item) but their spans will be changed since we applied formatting to them.
    let formatted_module = parse_file(formatted_input, path)?.value;

    // Actually find & insert the comments.
    add_comments(
        comment_map,
        unformatted_module,
        &formatted_module,
        formatted_code,
        unformatted_input,
    )
}

pub fn rewrite_with_comments<T: sway_parse::Parse + Format + LeafSpans>(
    formatter: &mut Formatter,
    unformatted_span: Span,
    formatted_code: &mut FormattedCode,
    last_formatted: usize,
) -> Result<(), FormatterError> {
    // Since we are adding comments into formatted code, in the next iteration the spans we find for the formatted code needs to be offsetted
    // as the total length of comments we added in previous iterations.
    let mut offset = 0;

    let mut to_rewrite = formatted_code[last_formatted..].to_string();

    let formatted_comment_spans = parse_snippet::<T>(&formatted_code[last_formatted..])
        .unwrap()
        .leaf_spans();
    let unformatted_comment_spans = parse_snippet::<T>(unformatted_span.as_str())
        .unwrap()
        .leaf_spans();

    // We will definetly have a span in the collected span since for a source code to be parsed there should be some tokens present.
    let mut previous_unformatted_comment_span = unformatted_comment_spans
        .first()
        .ok_or(FormatterError::CommentError)?;
    let mut previous_formatted_comment_span = formatted_comment_spans
        .first()
        .ok_or(FormatterError::CommentError)?;
    for (unformatted_comment_span, formatted_comment_span) in unformatted_comment_spans
        .iter()
        .zip(formatted_comment_spans.iter())
    {
        let start = previous_unformatted_comment_span.end + unformatted_span.start();
        let end = unformatted_comment_span.start + unformatted_span.start();
        let range = std::ops::Range { start, end };
        let iter = formatter.comments_context.map.comments_between(&range);

        let mut comments_found = vec![];
        for i in iter {
            comments_found.push(i.clone());
        }

        if !comments_found.is_empty() {
            // Since we're collecting extra newlines _between_ comments,
            // the first comment is always assumed to have 0 extra newlines.
            let mut extra_newlines = vec![0];
            if comments_found.len() > 1 {
                collect_extra_newlines(
                    &mut extra_newlines,
                    unformatted_span.clone(),
                    &comments_found,
                );
            }

            offset += insert_after_span_v2(
                previous_formatted_comment_span,
                comments_found.clone(),
                offset,
                &mut to_rewrite,
                extra_newlines,
            )?;

            formatter
                .comments_context
                .map
                .retain(|bs, _| !bs.contained_within(&range))
        }

        previous_unformatted_comment_span = unformatted_comment_span;
        previous_formatted_comment_span = formatted_comment_span;
    }

    formatted_code.truncate(last_formatted);
    write!(formatted_code, "{}", to_rewrite)?;
    Ok(())
}

fn collect_extra_newlines(
    extra_newlines: &mut Vec<usize>,
    unformatted_span: Span,
    comments_found: &Vec<Comment>,
) {
    // The first comment is always assumed to have no extra newlines.

    let mut prev_comment: Option<&Comment> = None;
    for comment in comments_found {
        if let Some(prev_comment) = prev_comment {
            let whitespace_between = unformatted_span.as_str()[prev_comment.span().end()
                - unformatted_span.start()
                ..comment.span().start() - unformatted_span.start()]
                .to_string();

            let mut extra_newlines_count =
                whitespace_between.chars().filter(|&c| c == '\n').count();

            extra_newlines_count -= 1;
            extra_newlines.push(extra_newlines_count);
        }

        prev_comment = Some(comment);
    }
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
    comment_map: &mut CommentMap,
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
            comment_map,
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

// A `CommentWithContext` is the `Comment` and the offset before and after it.
// A context is simply the chars between two points around a comment.
//
// The pre-context can be from the item we searched for this comment or from the last comment inside range.
//
// The post-context is an Option<String> that gets populated only if there is an "else" token following the `Comment`.
// It starts from end of the `Comment` and goes until the beginning of the `else` token.
// There may be other tokens we might need to look-ahead for in future.
#[derive(Debug)]
struct CommentWithContext {
    pre_context: String,
    comment: Comment,
    post_context: Option<String>,
}

/// Returns a list of comments between given spans. For each comment returns the Context
/// Context of a comment is basically the offset (the characters between the last item/comment) to the current comment
fn get_comments_between_spans(
    from: &ByteSpan,
    to: &ByteSpan,
    comment_map: &CommentMap,
    unformatted_code: &Arc<str>,
) -> Vec<CommentWithContext> {
    let mut comments_with_context: Vec<CommentWithContext> = Vec::new();
    if from < to {
        for (index, (comment_span, comment)) in
            comment_map.comments_in_range(from, to).iter().enumerate()
        {
            let starting_position_for_context = if index == 0 {
                // This is the first comment in the current range the context should be collected between from's end and comment's beginning
                from.end
            } else {
                // There is a comment before this one, so we should get the context starting from the last comment's end to the beginning of the current comment
                comments_with_context[index - 1].comment.span.end()
            };

            let mut rest_of_code = unformatted_code
                .get(comment_span.end..)
                .unwrap_or_default()
                .lines()
                .take(2);

            // consume '\n'
            let _ = rest_of_code.next();
            // actual next line of code that we're interested in
            let next_line = rest_of_code.next().unwrap_or_default();

            comments_with_context.push(CommentWithContext {
                pre_context: unformatted_code[starting_position_for_context..comment_span.start]
                    .to_string(),
                comment: comment.clone(),
                post_context: get_post_context(
                    unformatted_code,
                    starting_position_for_context,
                    comment_span,
                    next_line,
                ),
            });
        }
    }
    comments_with_context
}

fn format_context(context: &str, threshold: usize) -> String {
    let mut remaining_newlines = threshold;
    let mut formatted_context = String::new();
    for char in context.chars() {
        if char == '\n' {
            if remaining_newlines > 0 {
                formatted_context.push('\n');
                remaining_newlines -= 1;
            }
        } else {
            formatted_context.push(char);
        }
    }
    if formatted_context.starts_with("\n\n") {
        formatted_context.remove(0);
    }
    formatted_context
}

// In certain cases where comments come in between unusual places,
// ..
//     }
//     // This is a comment
//     else {
// ..
// We need to know the context after the comment as well.
fn get_post_context(
    unformatted_code: &Arc<str>,
    context_start: usize,
    comment_span: &ByteSpan,
    next_line: &str,
) -> Option<String> {
    if next_line.trim_start().starts_with("else") {
        // We want to align the 'else' token with the above comment, so this is just
        // same as the pre_context subtracted by 1.
        //
        // 1 here is somewhat a magic number. This is a result of the format_else_opt()
        // in utils/language/expr/conditional.rs always formatting with a whitespace.
        // We want to take that away since this 'else' will always be on a newline,
        // and will be overindented by one whitespace if left alone.
        Some(unformatted_code[context_start..(comment_span.start - 1)].to_string())
    } else {
        // If we don't find anything to format in the context after, we simply
        // return an empty context.
        None
    }
}

/// Inserts after given span and returns the offset. While inserting comments this also inserts contexts of the comments so that the alignment whitespaces/newlines are intact
fn insert_after_span(
    from: &ByteSpan,
    comments_to_insert: Vec<CommentWithContext>,
    offset: usize,
    formatted_code: &mut FormattedCode,
) -> Result<usize, FormatterError> {
    let iter = comments_to_insert.iter();
    let mut offset = offset;
    let mut comment_str = String::new();
    let mut pre_module_comment = false;
    for comment_with_context in iter {
        let CommentWithContext {
            pre_context,
            comment,
            post_context,
        } = comment_with_context;
        if comment.span.start() == from.start {
            pre_module_comment = true;
        }

        write!(
            comment_str,
            "{}{}{}",
            format_context(pre_context, 2),
            &format_comment(comment),
            format_context(
                post_context
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or_default(),
                2
            )
        )?;
    }
    let mut src_rope = Rope::from_str(formatted_code);
    // If the position we are going to be inserting from + 1 is a \n we are moving that \n after
    // this comment so if that is the case we are inserting after the \n
    if formatted_code.chars().nth(from.end + offset + 1) == Some('\n') {
        offset += 1;
    }

    if pre_module_comment {
        writeln!(comment_str)?;

        src_rope.insert(from.end + offset, comment_str.trim_start());
    } else {
        src_rope.insert(from.end + offset, &comment_str);
    }
    formatted_code.clear();
    formatted_code.push_str(&src_rope.to_string());
    Ok(comment_str.len())
}

fn is_empty_block(formatted_code: &mut FormattedCode, end: usize) -> bool {
    let substring = formatted_code[end..]
        .chars()
        .take_while(|&c| !c.is_whitespace())
        .count();

    formatted_code.chars().nth(end - 1) == Some('{')
        && (formatted_code.chars().nth(end) == Some('}')
            || formatted_code.chars().nth(end + substring + 1) == Some('}'))
}

/// Inserts after given span and returns the offset. While inserting comments this also inserts contexts of the comments so that the alignment whitespaces/newlines are intact
fn insert_after_span_v2(
    from: &ByteSpan,
    comments_to_insert: Vec<Comment>,
    offset: usize,
    formatted_code: &mut FormattedCode,
    extra_newlines: Vec<usize>,
) -> Result<usize, FormatterError> {
    let mut offset = offset;
    let mut comment_str = String::new();

    // We want to anchor the comment to the next line, and here,
    // we make the assumption here that comments will never be right before the final leaf span.
    let mut indent = formatted_code[from.end + offset..]
        .chars()
        .take_while(|c| c.is_whitespace())
        .collect::<String>();

    // There can be cases where comments are at the end.
    // If so, we try to search from before the end to find something to 'pin' to.
    if !is_empty_block(formatted_code, from.end) {
        if formatted_code.chars().nth(from.end + offset + indent.len()) == Some('}') {
            // It could be possible that the first comment found here is a Trailing,
            // then a Newlined.
            // We want all subsequent newlined comments to follow the indentation of the
            // previous line that is NOT a comment.

            if let Some(_) = comments_to_insert
                .iter()
                .find(|c| c.comment_kind == CommentKind::Newlined)
            {
                let prev_line = formatted_code[..from.end + offset]
                    .trim_end()
                    .chars()
                    .rev()
                    .take_while(|&c| c != '\n')
                    .collect::<String>();

                indent = prev_line
                    .chars()
                    .rev()
                    .take_while(|c| c.is_whitespace())
                    .collect();
                if let Some(comment) = comments_to_insert.first() {
                    if comment.comment_kind != CommentKind::Trailing {
                        comment_str.push('\n');
                    }
                }
            }
        }

        for (comment, extra_newlines) in comments_to_insert.iter().zip(extra_newlines) {
            // Check for newlines to preserve.
            for _ in 0..extra_newlines {
                comment_str.push('\n');
            }

            match comment.comment_kind {
                CommentKind::Trailing => {
                    if comments_to_insert.len() > 1 && indent.starts_with('\n') {
                        write!(comment_str, " {}", comment.span().as_str())?;
                    } else {
                        writeln!(comment_str, " {}", comment.span().as_str())?;
                    }
                }
                CommentKind::Newlined => {
                    if comments_to_insert.len() > 1 && indent.starts_with('\n') {
                        write!(comment_str, "{}{}", indent, comment.span().as_str())?;
                    } else {
                        writeln!(comment_str, "{}{}", indent, comment.span().as_str())?;
                    }
                }
                CommentKind::Inlined => {
                    if !formatted_code[..from.end].ends_with(' ') {
                        write!(comment_str, " ")?;
                    }
                    write!(comment_str, "{}", comment.span().as_str())?;
                    if !formatted_code[from.end + offset..].starts_with([' ', '\n']) {
                        write!(comment_str, " ")?;
                    }
                }
                CommentKind::Multilined => {
                    write!(comment_str, "{}{}", indent, comment.span().as_str())?;
                }
            };
        }
    }

    let mut src_rope = Rope::from_str(formatted_code);

    if formatted_code.chars().nth(from.end + offset + 1) == Some('\n') {
        offset += 1;
    }

    if let Some(char) = src_rope.get_char(from.end + offset) {
        if char == '\n' && comment_str.ends_with('\n') {
            comment_str.pop();
        }
    };
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
    use super::ByteSpan;
    use crate::utils::map::comments::{CommentMap, CommentRange};
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
        let map = CommentMap::from_src(Arc::from(input)).unwrap();
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
        let map = CommentMap::from_src(Arc::from(input)).unwrap();
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
        let map = CommentMap::from_src(Arc::from(input)).unwrap();
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
        let map = CommentMap::from_src(Arc::from(input)).unwrap();
        assert!(!map.is_empty());
        let found_comments = map.comments_in_range(&range_start_span, &range_end_span);
        assert_eq!(found_comments[0].1.span.as_str(), "// test");
    }
}
