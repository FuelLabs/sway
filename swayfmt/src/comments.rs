use ropey::Rope;
use std::{fmt::Write, ops::Range};
use sway_ast::token::{Comment, CommentKind};
use sway_types::{Span, Spanned};

use crate::{
    formatter::FormattedCode,
    parse::parse_snippet,
    utils::map::{
        byte_span::{ByteSpan, LeafSpans},
        comments::CommentMap,
    },
    Format, Formatter, FormatterError,
};

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
/// removes comments that are written here from the CommentMap for later use.
///
/// Most comment formatting should be done using `rewrite_with_comments` in
/// the context of the AST, but in some cases (eg. at the end of module) we require this function.
///
/// Returns:
/// `Ok(true)` on successful execution with comments written,
/// `Ok(false)` on successful execution and if there are no comments within the given range,
/// `Err` if a FormatterError was encountered.
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
        if formatted_code.ends_with(&['{', '}']) && !formatted_code.ends_with('\n') {
            writeln!(formatted_code)?;
        }

        for comment in comments_iter {
            let newlines = collect_newlines_after_comment(&formatter.comments_context, comment);

            match comment.comment_kind {
                CommentKind::Newlined => {
                    write!(
                        formatted_code,
                        "{}{}{}",
                        formatter.shape.indent.to_string(&formatter.config)?,
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
                    write!(formatted_code, " {} ", comment.span().as_str(),)?;
                }
                CommentKind::Multilined => {}
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

/// Adds the comments from comment_map to correct places in the formatted code. This requires us
/// both the unformatted and formatted code's modules as they will have different spans for their
/// visitable positions. While traversing the unformatted module, `add_comments` searches for comments. If there is a comment found
/// places the comment to the correct place at formatted_code.
///
/// This requires both the unformatted_code itself and the parsed version of it, because
/// unformatted_code is used for context lookups and unformatted_module is required for actual
/// traversal. When `add_comments` is called we have already parsed the unformatted_code so there is no need
/// to parse it again.
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
            collect_extra_newlines(
                &mut extra_newlines,
                unformatted_span.clone(),
                &comments_found,
            );

            offset += insert_after_span(
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
    write!(formatted_code, "{to_rewrite}")?;
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

            if extra_newlines_count > 1 {
                // If there is a bunch of newlines, we always want to collapse it to 1.
                extra_newlines_count = 1;
            } else {
                extra_newlines_count = 0;
            }
            extra_newlines.push(extra_newlines_count);
        }

        prev_comment = Some(comment);
    }
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
fn insert_after_span(
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

            if comments_to_insert
                .iter()
                .any(|c| c.comment_kind == CommentKind::Newlined)
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

#[cfg(test)]
mod tests {
    use crate::utils::map::byte_span::ByteSpan;
    use std::sync::Arc;

    use super::*;

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
            CommentMap::from_src(Arc::from(commented_code)).unwrap(),
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
            CommentMap::from_src(Arc::from(multiline_comment)).unwrap(),
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
            CommentMap::from_src(Arc::from(multi_newline_comments)).unwrap(),
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
