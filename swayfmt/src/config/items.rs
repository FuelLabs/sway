//! Configuration options related to item formatting.

use crate::{
    config::user_opts::ItemsOptions,
    constants::{DEFAULT_BLANK_LINES_LOWER_BOUND, DEFAULT_BLANK_LINES_UPPER_BOUND},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone)]
pub struct Items {
    /// Brace style for items.
    pub item_brace_style: ItemBraceStyle,
    /// Maximum number of blank lines which can be put between items.
    pub blank_lines_upper_bound: usize,
    /// Minimum number of blank lines which must be put between items.
    pub blank_lines_lower_bound: usize,
    /// Put empty-body functions and impls on a single line.
    pub empty_item_single_line: bool,
}

impl Default for Items {
    fn default() -> Self {
        Self {
            item_brace_style: Default::default(),
            blank_lines_upper_bound: DEFAULT_BLANK_LINES_UPPER_BOUND,
            blank_lines_lower_bound: DEFAULT_BLANK_LINES_LOWER_BOUND,
            empty_item_single_line: true,
        }
    }
}

impl Items {
    pub fn from_opts(opts: &ItemsOptions) -> Self {
        let default = Self::default();
        Self {
            item_brace_style: opts.item_brace_style.unwrap_or(default.item_brace_style),
            blank_lines_upper_bound: opts
                .blank_lines_upper_bound
                .unwrap_or(default.blank_lines_upper_bound),
            blank_lines_lower_bound: opts
                .blank_lines_lower_bound
                .unwrap_or(default.blank_lines_lower_bound),
            empty_item_single_line: opts
                .empty_item_single_line
                .unwrap_or(default.empty_item_single_line),
        }
    }
}

/// Preference of how list-like items are displayed.
///
/// Defaults to `Tall`.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum ItemsLayout {
    /// Fit as much on one line as possible.
    Compressed,
    /// Items are placed horizontally if sufficient space, vertically otherwise.
    #[default]
    Tall,
    /// Place every item on a separate line.
    Vertical,
}

/// Where to put the opening brace of items (`fn`, `impl`, etc.).
///
/// Defaults to `SameLineWhere`.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum ItemBraceStyle {
    /// Put the opening brace on the next line.
    AlwaysNextLine,
    /// Put the opening brace on the same line, if possible.
    PreferSameLine,
    /// Prefer the same line except where there is a where-clause, in which
    /// case force the brace to be put on the next line.
    #[default]
    SameLineWhere,
}
