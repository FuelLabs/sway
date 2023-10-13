//! Configuration options related to formatting comments.
use crate::{config::user_opts::CommentsOptions, constants::DEFAULT_MAX_COMMENT_WIDTH};

#[derive(Debug, Clone)]
pub struct Comments {
    /// Break comments to fit on the line.
    /// Defaults to `false`.
    pub wrap_comments: bool,
    /// Maximum length of comments. No effect unless wrap_comments = true.
    /// Defaults to `80`.
    pub comment_width: usize,
    /// Convert /* */ comments to // comments where possible
    /// Defaults to `false`.
    pub normalize_comments: bool,
}

impl Default for Comments {
    fn default() -> Self {
        Self {
            wrap_comments: false,
            comment_width: DEFAULT_MAX_COMMENT_WIDTH,
            normalize_comments: false,
        }
    }
}

impl Comments {
    pub fn from_opts(opts: &CommentsOptions) -> Self {
        let default = Self::default();
        Self {
            wrap_comments: opts.wrap_comments.unwrap_or(default.wrap_comments),
            comment_width: opts.comment_width.unwrap_or(default.comment_width),
            normalize_comments: opts
                .normalize_comments
                .unwrap_or(default.normalize_comments),
        }
    }
}
