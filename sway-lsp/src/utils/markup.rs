//! Markdown formatting.
//!
//! Sometimes, we want to display a "rich text" in the UI. At the moment, we use
//! markdown for this purpose.
//! Modified from rust-analyzer.
use std::fmt;

/// A handy wrapper around `String` for constructing markdown documents.
#[derive(Default, Debug)]
pub struct Markup {
    text: String,
}

impl From<Markup> for String {
    fn from(markup: Markup) -> Self {
        markup.text
    }
}

impl From<String> for Markup {
    fn from(text: String) -> Self {
        Markup { text }
    }
}

impl fmt::Display for Markup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.text, f)
    }
}

impl Markup {
    /// Get the inner string as a str.
    pub fn as_str(&self) -> &str {
        self.text.as_str()
    }
    /// Contents will be formatted with sway syntax highlighting.
    pub fn fenced_block(contents: &impl fmt::Display) -> Markup {
        format!("```sway\n{}\n```", contents).into()
    }
}
