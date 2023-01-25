use std::{fmt::Write, ops::Range};
use sway_ast::token::Comment;
use sway_types::Spanned;

use crate::{formatter::FormattedCode, Formatter, FormatterError};

// Given a start, an end and a CommentMap, return references to all comments
// contained within a given start and end of a span, in an exclusive range.
pub fn comments_between<'a>(
    range: &'a Range<usize>,
    formatter: &'a Formatter,
) -> impl Iterator<Item = &'a Comment> {
    formatter.comment_map.iter().filter_map(|(bs, c)| {
        if bs.start > range.start && bs.end < range.end {
            Some(c)
        } else {
            None
        }
    })
}

// Writes comments between a given start and end and removes them from the formatter's CommentMap.
// This returns Ok(true) on successful execution AND comments were written, but returns
// Ok(false) on successful execution without any comments written. Returns Err on failure.
pub fn maybe_write_comments_from_map(
    formatted_code: &mut FormattedCode,
    range: Range<usize>,
    formatter: &mut Formatter,
) -> Result<bool, FormatterError> {
    {
        let mut comments_iter = comments_between(&range, formatter).enumerate().peekable();

        if comments_iter.peek().is_none() {
            return Ok(false);
        };

        for (i, comment) in comments_iter {
            // Write comments on a newline (for now). New behavior might be required
            // to support trailing comments.
            if i == 0 {
                writeln!(formatted_code)?;
            }
            writeln!(
                formatted_code,
                "{}{}",
                formatter.shape.indent.to_string(&formatter.config)?,
                comment.span().as_str(),
            )?;
        }
    }

    formatter
        .comment_map
        .retain(|bs, _| !(bs.start > range.start && bs.end < range.end));

    Ok(true)
}
