use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct SwayFormatConfig {
    // TODO: make default 100
    max_width: usize, // Max width of each line
    hard_tabs: bool, // Use tab characters for indentation, spaces for alignment
    // TODO: make default 4
    tab_spaces: usize, // Number of spaces per tab
    // TODO: make default Auto
    newline_style: NewlineStyle, // Unix or Windows line endings
    // TODO: make default Block
    indent_style: IndentStyle, // How we indent expressions or items
}

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

#[derive(Serialize, Deserialize, Debug)]
pub enum IndentStyle {
    /// First line on the same line as the opening brace, all lines aligned with
    /// the first line.
    Visual,
    /// First line is on a new line and all lines align with **block** indent.
    Block,
}