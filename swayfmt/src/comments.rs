use std::fmt::Write;
use sway_ast::token::Comment;
use sway_types::Spanned;

use crate::{formatter::FormattedCode, Formatter, FormatterError};

// Given a Span and a CommentMap, return all comments contained
// in the span.
pub fn get_comments_between(start: usize, end: usize, formatter: &mut Formatter) -> Vec<&Comment> {
    let mut comments = vec![];
    let iter = formatter.comment_map.clone().into_keys();

    for bs in iter {
        if bs.start > start && bs.end < end {
            if let Some(comment) = formatter.comment_map.get(&bs) {
                comments.push(comment)
            }
        }
    }

    comments
}

// Given a Span and a CommentMap, return all comments contained
// in the span.
pub fn take_comments_between(start: usize, end: usize, formatter: &mut Formatter) -> Vec<Comment> {
    let mut comments = vec![];
    let iter = formatter.comment_map.clone().into_keys();
    println!("finding comments between {} and {} ", start, end);

    for bs in iter {
        if bs.start > start && bs.end < end {
            if let Some(comment) = formatter.comment_map.remove(&bs) {
                comments.push(comment)
            }
        }
    }

    comments
}

pub fn maybe_write_comments_from_map(
    to: &mut FormattedCode,
    start: usize,
    end: usize,
    formatter: &mut Formatter,
) -> Result<bool, FormatterError> {
    let comments = take_comments_between(start, end, formatter);
    if !comments.is_empty() {
        writeln!(to)?;
        for comment in comments {
            writeln!(
                to,
                "{}{}",
                formatter.shape.indent.to_string(&formatter.config)?,
                comment.span().as_str(),
            )?;
        }
    } else {
        return Ok(false);
    }

    Ok(true)
}
