use crate::{
    formatter::FormattedCode,
    parse::parse_snippet,
    utils::map::{
        byte_span::{ByteSpan, LeafSpans},
        comments::CommentMap,
    },
    Format, Formatter, FormatterError,
};
use ropey::Rope;
use std::{fmt::Write, ops::Range};
use sway_ast::token::{Comment, CommentKind};
use sway_types::{Span, Spanned};

pub type UnformattedCode = String;

#[derive(Debug, Default, Clone)]
pub struct CommentsContext {
    /// A BTreeMap of the mapping ByteSpan->Comment for inserting comments.
    pub map: CommentMap,
    /// Original unformatted code that the formatter tries to format.
    /// The Formatter requires this to preserve newlines between comments.
    unformatted_code: UnformattedCode,
}

impl CommentsContext {
    pub fn new(map: CommentMap, unformatted_code: UnformattedCode) -> Self {
        Self {
            map,
            unformatted_code,
        }
    }
    pub fn unformatted_code(&self) -> &str {
        &self.unformatted_code
    }
}

#[inline]
pub fn has_comments_in_formatter(formatter: &Formatter, range: &Range<usize>) -> bool {
    formatter
        .comments_context
        .map
        .comments_between(range)
        .peekable()
        .peek()
        .is_some()
}

#[inline]
pub fn has_comments<I: Iterator>(comments: I) -> bool {
    comments.peekable().peek().is_some()
}

/// This function collects newlines to insert after a given Comment span to preserve them.
pub fn collect_newlines_after_comment(
    comments_context: &CommentsContext,
    comment: &Comment,
) -> String {
    comments_context.unformatted_code()[comment.span().end()..]
        .chars()
        .take_while(|&c| c.is_whitespace())
        .filter(|&c| c == '\n')
        .collect()
}

/// Writes a trailing comment using potentially destructive 'truncate()' to strip the end
/// of whitespaces.
fn write_trailing_comment(
    formatted_code: &mut FormattedCode,
    comment: &Comment,
) -> Result<(), FormatterError> {
    formatted_code.truncate(formatted_code.trim_end().len());
    writeln!(formatted_code, " {}", comment.span().as_str().trim_end())?;

    Ok(())
}

/// Given a range, writes comments contained within the range. This function
/// removes comments that are written here from the [CommentMap] for later use.
///
/// Most comment formatting should be done using [rewrite_with_comments] in
/// the context of the AST, but in some cases (eg. at the end of module) we require this function.
///
/// Returns:
/// `Ok(true)` on successful execution with comments written,
/// `Ok(false)` on successful execution and if there are no comments within the given range,
/// `Err` if a FormatterError was encountered.
///
/// The `range` can be an empty [Range], or have its start being greater then its end.
/// This is to support formatting arbitrary lexed trees, that are not necessarily backed by source code.
pub fn write_comments(
    formatted_code: &mut FormattedCode,
    range: Range<usize>,
    formatter: &mut Formatter,
) -> Result<bool, FormatterError> {
    {
        let mut comments_iter = formatter
            .comments_context
            .map
            .comments_between(&range)
            .peekable();

        if comments_iter.peek().is_none() {
            return Ok(false);
        };

        // If the already formatted code ends with some pattern and doesn't already end with a newline,
        // we want to add a newline here.
        if formatted_code.ends_with(['{', '}']) && !formatted_code.ends_with('\n') {
            writeln!(formatted_code)?;
        }

        for comment in comments_iter {
            let newlines = collect_newlines_after_comment(&formatter.comments_context, comment);

            match comment.comment_kind {
                CommentKind::Newlined => {
                    write!(
                        formatted_code,
                        "{}{}{}",
                        formatter.indent_to_str()?,
                        comment.span().as_str(),
                        newlines
                    )?;
                }
                CommentKind::Trailing => {
                    write_trailing_comment(formatted_code, comment)?;
                }
                CommentKind::Inlined => {
                    // We do a trim and truncate here to ensure that only a single whitespace separates
                    // the inlined comment from the previous token.
                    formatted_code.truncate(formatted_code.trim_end().len());
                    write!(formatted_code, " {} ", comment.span().as_str())?;
                }
                CommentKind::Multilined => {
                    write!(
                        formatted_code,
                        "{}{}",
                        formatter.indent_to_str()?,
                        comment.span().as_str(),
                    )?;
                }
            }
        }
    }

    // Keep comments that are NOT within `range` within the CommentMap.
    // This is destructive behavior for comments since if any steps above fail
    // and comments were not written, `retains()` will still delete these comments.
    formatter
        .comments_context
        .map
        .retain(|bs, _| !bs.contained_within(&range));

    Ok(true)
}

/// The main function that rewrites a piece of formatted code with comments found in its unformatted version.
///
/// This takes a given AST node's unformatted span, its leaf spans and its formatted code (a string) and
/// parses the equivalent formatted version to get its leaf spans. We traverse the spaces between both
/// formatted and unformatted leaf spans to find possible comments and inserts them between.
///
/// The `unformatted_span` can be an empty [Span]. This is to support formatting arbitrary lexed trees,
/// that are not necessarily backed by source code.
pub fn rewrite_with_comments<T: sway_parse::Parse + Format + LeafSpans>(
    formatter: &mut Formatter,
    unformatted_span: Span,
    unformatted_leaf_spans: Vec<ByteSpan>,
    formatted_code: &mut FormattedCode,
    last_formatted: usize,
) -> Result<(), FormatterError> {
    // Since we are adding comments into formatted code, in the next iteration the spans we find for the formatted code needs to be offsetted
    // as the total length of comments we added in previous iterations.
    let mut offset = 0;
    let mut to_rewrite = formatted_code[last_formatted..].to_string();

    let formatted_leaf_spans =
        parse_snippet::<T>(&formatted_code[last_formatted..], formatter.experimental)?.leaf_spans();

    let mut previous_unformatted_leaf_span = unformatted_leaf_spans
        .first()
        .ok_or(FormatterError::CommentError)?;
    let mut previous_formatted_leaf_span = formatted_leaf_spans
        .first()
        .ok_or(FormatterError::CommentError)?;
    for (unformatted_leaf_span, formatted_leaf_span) in unformatted_leaf_spans
        .iter()
        .zip(formatted_leaf_spans.iter())
    {
        // Search for comments between the previous leaf span's end and the next leaf span's start
        let range = std::ops::Range {
            start: previous_unformatted_leaf_span.end,
            end: unformatted_leaf_span.start,
        };
        let iter = formatter.comments_context.map.comments_between(&range);

        let mut comments_found = vec![];
        for i in iter {
            comments_found.push(i.clone());
        }

        if !comments_found.is_empty() {
            let extra_newlines = collect_extra_newlines(unformatted_span.clone(), &comments_found);

            offset += insert_after_span(
                previous_formatted_leaf_span,
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

        previous_unformatted_leaf_span = unformatted_leaf_span;
        previous_formatted_leaf_span = formatted_leaf_span;
    }

    formatted_code.truncate(last_formatted);
    write!(formatted_code, "{to_rewrite}")?;
    Ok(())
}

/// Collect extra newline before comment(s). The main purpose of this function is to maintain
/// newlines between comments when inserting multiple comments at once.
fn collect_extra_newlines(unformatted_span: Span, comments_found: &Vec<Comment>) -> Vec<usize> {
    // The first comment is always assumed to have no extra newlines before itself.
    let mut extra_newlines = vec![0];

    if comments_found.len() == 1 {
        return extra_newlines;
    }

    let mut prev_comment: Option<&Comment> = None;
    for comment in comments_found {
        if let Some(prev_comment) = prev_comment {
            // Get whitespace between the end of the previous comment and the start of the next.
            let whitespace_between = unformatted_span.as_str()[prev_comment.span().end()
                - unformatted_span.start()
                ..comment.span().start() - unformatted_span.start()]
                .to_string();

            // Count the number of newline characters we found above.
            // By default, we want 0 extra newlines, but if there are more than 1 extra newline, we want to squash it to 1.
            let mut extra_newlines_count = 0;
            if whitespace_between.chars().filter(|&c| c == '\n').count() > 1 {
                extra_newlines_count = 1;
            };

            extra_newlines.push(extra_newlines_count);
        }

        prev_comment = Some(comment);
    }

    extra_newlines
}

/// Check if a block is empty. When formatted without comments, empty code blocks are formatted into "{}", which is what this check is for.
fn is_empty_block(formatted_code: &FormattedCode, end: usize) -> bool {
    formatted_code.chars().nth(end - 1) == Some('{') && formatted_code.chars().nth(end) == Some('}')
}

/// Main driver of writing comments. This function is a no-op if the block of code is empty.
///
/// This iterates through comments inserts each of them after a given span and returns the offset.
/// While inserting comments this also inserts whitespaces/newlines so that alignment is intact.
/// To do the above, there are some whitespace heuristics we stick to:
///
/// 1) Assume comments are anchored to the line below, and follow its alignment.
/// 2) In some cases the line below is the end of the function eg. it contains only a closing brace '}'.
///    in such cases we then try to anchor the comment to the line above.
/// 3) In the cases of entirely empty blocks we actually should prefer using `write_comments` over
///    `rewrite_with_comments` since `write_comments` would have the formatter's indentation context.
fn insert_after_span(
    from: &ByteSpan,
    comments_to_insert: Vec<Comment>,
    offset: usize,
    formatted_code: &mut FormattedCode,
    extra_newlines: Vec<usize>,
) -> Result<usize, FormatterError> {
    let mut comment_str = String::new();

    // We want to anchor the comment to the next line, and here,
    // we make the assumption here that comments will never be right before the final leaf span.
    let mut indent = formatted_code[from.end + offset..]
        .chars()
        .take_while(|c| c.is_whitespace())
        .collect::<String>();

    // In the case of empty blocks, we do not know the indentation of comments at that time.
    // Writing comments in empty blocks has to be deferred to `write_comments` instead, which will
    // contain the Formatter's indentation context.
    if !is_empty_block(formatted_code, from.end) {
        // There can be cases where comments are at the end.
        // If so, we try to 'pin' our comment's indentation to the previous line instead.
        if formatted_code.chars().nth(from.end + offset + indent.len()) == Some('}') {
            // It could be possible that the first comment found here is a Trailing,
            // then a Newlined.
            // We want all subsequent newlined comments to follow the indentation of the
            // previous line that is NOT a comment.
            if comments_to_insert
                .iter()
                .any(|c| c.comment_kind == CommentKind::Newlined)
            {
                // Find and assign the indentation of the previous line to `indent`.
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

        let mut src_rope = Rope::from_str(formatted_code);

        // We do a sanity check here to ensure that we don't insert an extra newline
        // if the place at which we're going to insert comments already ends with '\n'.
        if let Some(char) = src_rope.get_char(from.end + offset) {
            if char == '\n' && comment_str.ends_with('\n') {
                comment_str.pop();
            }
        };

        // Insert the actual comment(s).
        src_rope
            .try_insert(from.end + offset, &comment_str)
            .map_err(|_| FormatterError::CommentError)?;

        formatted_code.clear();
        formatted_code.push_str(&src_rope.to_string());
    }

    // In order to handle special characters, we return the number of characters rather than
    // the size of the string.
    Ok(comment_str.chars().count())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::map::byte_span::ByteSpan;

    /// For readability of the assertions, the comments written within these snippets will be the
    /// ByteSpan representations instead of some random comment,
    /// eg. the below '// 10-18' comment is representative of its start (10) and end (18) index
    /// This way we have contextual knowledge of what the source code looks like when we
    /// do 'comments_ctx.map.get(&ByteSpan { start: 10, end: 18 })'.
    /// If more tests are to be added here, it is highly encouraged to follow this convention.
    #[test]
    fn test_collect_newlines_after_comment() {
        let commented_code = r#"contract;
// 10-18
pub fn main() -> bool {
    true
}
"#;
        let mut comments_ctx = CommentsContext::new(
            CommentMap::from_src(commented_code.into()).unwrap(),
            commented_code.to_string(),
        );
        assert_eq!(
            collect_newlines_after_comment(
                &comments_ctx,
                comments_ctx
                    .map
                    .get(&ByteSpan { start: 10, end: 18 })
                    .unwrap(),
            ),
            "\n"
        );

        let multiline_comment = r#"contract;
pub fn main() -> bool {
    // 38-46
    // 51-59
    true
}
"#;

        comments_ctx = CommentsContext::new(
            CommentMap::from_src(multiline_comment.into()).unwrap(),
            multiline_comment.to_string(),
        );

        assert_eq!(
            collect_newlines_after_comment(
                &comments_ctx,
                comments_ctx
                    .map
                    .get(&ByteSpan { start: 38, end: 46 })
                    .unwrap(),
            ),
            "\n"
        );

        let multi_newline_comments = r#"contract;
pub fn main() -> bool {
    // 38-46

    // 52-60
    true
}
"#;

        comments_ctx = CommentsContext::new(
            CommentMap::from_src(multi_newline_comments.into()).unwrap(),
            multi_newline_comments.to_string(),
        );

        assert_eq!(
            collect_newlines_after_comment(
                &comments_ctx,
                comments_ctx
                    .map
                    .get(&ByteSpan { start: 38, end: 46 })
                    .unwrap(),
            ),
            "\n\n"
        );
    }
}
