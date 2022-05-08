//! Configuration options related to formatting comments.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Comments {
    /// Break comments to fit on the line.
    pub wrap_comments: bool,
    /// Maximum length of comments. No effect unless wrap_comments = true.
    pub comment_width: usize,
    /// Convert /* */ comments to // comments where possible
    pub normalize_comments: bool,
}
