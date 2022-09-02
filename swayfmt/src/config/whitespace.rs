//! Standard system and editor whitespace configuration options. Advanced whitespace options will be deferred to their corresponding sub-classes.
use crate::{
    config::user_opts::WhitespaceOptions,
    constants::{
        CARRIAGE_RETURN, DEFAULT_MAX_LINE_WIDTH, DEFAULT_NEWLINE_THRESHOLD, DEFAULT_TAB_SPACES,
        LINE_FEED,
    },
};
use serde::{Deserialize, Serialize};

/// Whitespace styling preferences.
#[derive(Debug, Copy, Clone)]
pub struct Whitespace {
    /// Maximum width of each line.
    pub max_width: usize,
    /// Use tab characters for indentation, spaces for alignment.
    pub hard_tabs: bool,
    /// Number of spaces per tab.
    pub tab_spaces: usize,
    /// Unix or Windows line endings.
    pub newline_style: NewlineStyle,
    /// How we indent expressions or items.
    pub indent_style: IndentStyle,
    /// Max number of newlines allowed between statements before collapsing them to threshold
    pub newline_threshold: usize,
}

impl Default for Whitespace {
    fn default() -> Self {
        Self {
            max_width: DEFAULT_MAX_LINE_WIDTH,
            hard_tabs: false,
            tab_spaces: DEFAULT_TAB_SPACES,
            newline_style: NewlineStyle::Auto,
            indent_style: IndentStyle::Block,
            newline_threshold: DEFAULT_NEWLINE_THRESHOLD,
        }
    }
}

impl Whitespace {
    pub fn from_opts(opts: &WhitespaceOptions) -> Self {
        let default = Self::default();
        Self {
            max_width: opts.max_width.unwrap_or(default.max_width),
            hard_tabs: opts.hard_tabs.unwrap_or(default.hard_tabs),
            tab_spaces: opts.tab_spaces.unwrap_or(default.tab_spaces),
            newline_style: opts.newline_style.unwrap_or(default.newline_style),
            indent_style: opts.indent_style.unwrap_or(default.indent_style),
            newline_threshold: opts.newline_threshold.unwrap_or(default.newline_threshold),
        }
    }
}

/// Handling of line indentation for expressions or items.
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum IndentStyle {
    /// First line on the same line as the opening brace, all lines aligned with
    /// the first line.
    Visual,
    /// First line is on a new line and all lines align with **block** indent.
    Block,
}

/// Handling of which OS new-line style should be applied.
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
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

/// The definitive system type for `[NewlineStyle]`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NewlineSystemType {
    Windows,
    Unix,
}

impl NewlineSystemType {
    pub fn get_newline_style(newline_style: NewlineStyle, raw_input_text: &str) -> Self {
        match newline_style {
            NewlineStyle::Auto => Self::auto_detect_newline_style(raw_input_text),
            NewlineStyle::Native => Self::native_newline_style(),
            NewlineStyle::Windows => Self::Windows,
            NewlineStyle::Unix => Self::Unix,
        }
    }

    pub fn auto_detect_newline_style(raw_input_text: &str) -> Self {
        let first_line_feed_pos = raw_input_text.chars().position(|ch| ch == LINE_FEED);
        match first_line_feed_pos {
            Some(first_line_feed_pos) => {
                let char_before_line_feed_pos = first_line_feed_pos.saturating_sub(1);
                let char_before_line_feed = raw_input_text.chars().nth(char_before_line_feed_pos);
                match char_before_line_feed {
                    Some(CARRIAGE_RETURN) => Self::Windows,
                    _ => Self::Unix,
                }
            }
            None => Self::native_newline_style(),
        }
    }

    fn native_newline_style() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else {
            Self::Unix
        }
    }
}
