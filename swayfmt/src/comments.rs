use std::{fmt::Write, ops::Range};
use sway_ast::token::CommentKind;
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
    pub unformatted_code: UnformattedCode,
}

impl CommentsContext {
    pub fn unformatted_code(&self) -> &str {
        &self.unformatted_code
    }
}

pub fn has_comments<I: Iterator>(comments: I) -> bool {
    comments.peekable().peek().is_some()
}

pub fn gather_newlines(src: &str) -> String {
    src.chars()
        .take_while(|&c| c.is_whitespace())
        .filter(|&c| c == '\n')
        .collect()
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

        while let Some(comment) = comments_iter.next() {
            let newlines = gather_newlines(
                &formatter.comments_context.unformatted_code()[comment.span().end()..],
            );

            // Write comments on a newline (for now). New behavior might be required
            // to support trailing comments.
            if formatted_code.trim_end().ends_with(&[']', ';']) {
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
                        formatted_code.truncate(formatted_code.trim_end().len());
                        writeln!(formatted_code, " {}", comment.span().as_str().trim_end())?;
                        if comments_iter.peek().is_none() {
                            write!(
                                formatted_code,
                                "{}",
                                formatter.shape.indent.to_string(&formatter.config)?,
                            )?;
                        }
                    }
                }
            } else {
                writeln!(
                    formatted_code,
                    "{}{}",
                    formatter.shape.indent.to_string(&formatter.config)?,
                    comment.span().as_str(),
                )?;
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
