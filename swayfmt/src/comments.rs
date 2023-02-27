use std::{fmt::Write, ops::Range};
use sway_ast::token::{Comment, CommentKind};
use sway_types::Spanned;

use crate::{
    formatter::FormattedCode, utils::map::comments::CommentMap, Formatter, FormatterError,
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
