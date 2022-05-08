//! Basic configuration options.
use serde::{Deserialize, Serialize};

/// Basic styling preferences.
#[derive(Serialize, Deserialize, Debug)]
pub struct Fundamentals {
    // TODO: make default 100
    /// Maximum width of each line.
    pub max_width: usize,
    // TODO: make default false
    /// Use tab characters for indentation, spaces for alignment.
    pub hard_tabs: bool,
    // TODO: make default 4
    /// Number of spaces per tab.
    pub tab_spaces: usize,
    // TODO: make default Auto
    /// Unix or Windows line endings.
    pub newline_style: NewlineStyle,
    // TODO: make default Block
    /// How we indent expressions or items.
    pub indent_style: IndentStyle,
}

/// Handling of which OS new-line style should be applied.
#[derive(Serialize, Deserialize, Debug)]
pub enum NewlineStyle {
    /// Auto-detect based on the raw source input.
    Auto,
    /// Force CRLF (`\r\n`).
    Windows,
    /// Force CR (`\n).
    Unix,
    /// `\r\n` in Windows, `\n` on other platforms.
    Native,
}

/// Handling of line indentation for expressions or items.
#[derive(Serialize, Deserialize, Debug)]
pub enum IndentStyle {
    /// First line on the same line as the opening brace, all lines aligned with
    /// the first line.
    Visual,
    /// First line is on a new line and all lines align with **block** indent.
    Block,
}
