use std::{fmt::Write, ops::Range};
use sway_types::Spanned;

use crate::{formatter::FormattedCode, Formatter, FormatterError};

pub fn has_comments<I: Iterator>(comments: I) -> bool {
    comments.peekable().peek().is_some()
}

/// Given a range, writes comments contained within the range. This function
/// removes comments that are written here from the CommentMap for later use.
///
/// Returns:
/// `Ok(true)` on successful execution with comments written,
/// `Ok(false)` on successful execution and if there are no comments within the given range,
/// `Err` if a FormatterError was encountered.
pub fn maybe_write_comments_from_map(
    formatted_code: &mut FormattedCode,
    range: Range<usize>,
    formatter: &mut Formatter,
) -> Result<bool, FormatterError> {
    {
        let mut comments_iter = formatter.comment_map.comments_between(&range).peekable();

        if comments_iter.peek().is_none() {
            return Ok(false);
        };

        // If the already formatted code ends with some pattern and doesn't already end with a newline,
        // we want to add a newline here.
        if formatted_code.ends_with(&['{', '}']) && !formatted_code.ends_with('\n') {
            writeln!(formatted_code)?;
        }

        for comment in comments_iter {
            // Write comments on a newline (for now). New behavior might be required
            // to support trailing comments.
            writeln!(
                formatted_code,
                "{}{}",
                formatter.shape.indent.to_string(&formatter.config)?,
                comment.span().as_str(),
            )?;
        }
    }

    // Keep comments that are NOT within `range` within the CommentMap.
    // This is destructive behavior for comments since if any steps above fail
    // and comments were not written, `retains()` will still delete these comments.
    formatter
        .comment_map
        .retain(|bs, _| !bs.contained_within(&range));

    Ok(true)
}
