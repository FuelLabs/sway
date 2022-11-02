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
    /// Creates a new empty `Markup`.
    pub fn new() -> Self {
        Self {
            text: String::new(),
        }
    }

    pub fn maybe_add_sway_block(self, contents: Option<String>) -> Self {
        match contents {
            Some(contents) => self.fenced_sway_block(&contents).line_sperator(),
            None => self,
        }
    }

    /// Contents will be formatted with sway syntax highlighting.
    pub fn fenced_sway_block(mut self, contents: &impl fmt::Display) -> Self {
        let code_block = format!("```sway\n{}\n```", contents);
        self.text.push_str(&code_block);
        self
    }

    /// Add a new line.
    pub fn line_sperator(mut self) -> Self {
        self.text.push_str("\n---\n");
        self
    }

    /// Add text to the markup.
    pub fn text(mut self, contents: &str) -> Self {
        self.text.push_str(contents);
        self
    }

    /// Get the inner string as a str.
    pub fn as_str(&self) -> &str {
        self.text.as_str()
    }
}
