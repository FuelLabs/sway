//! Configuration options related to item formatting.
use serde::{Deserialize, Serialize};

use super::{lists::ListTactic, user_opts::ItemsOptions};
use crate::constants::{DEFAULT_BLANK_LINES_LOWER_BOUND, DEFAULT_BLANK_LINES_UPPER_BOUND};

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
            item_brace_style: ItemBraceStyle::SameLineWhere,
            blank_lines_upper_bound: DEFAULT_BLANK_LINES_UPPER_BOUND,
            blank_lines_lower_bound: DEFAULT_BLANK_LINES_LOWER_BOUND,
            empty_item_single_line: true,
        }
    }
}

impl Items {
    pub fn from_opts(opts: &ItemsOptions) -> Self {
        Self {
            item_brace_style: opts
                .item_brace_style
                .unwrap_or(ItemBraceStyle::SameLineWhere),
            blank_lines_upper_bound: opts
                .blank_lines_upper_bound
                .unwrap_or(DEFAULT_BLANK_LINES_UPPER_BOUND),
            blank_lines_lower_bound: opts
                .blank_lines_lower_bound
                .unwrap_or(DEFAULT_BLANK_LINES_LOWER_BOUND),
            empty_item_single_line: opts.empty_item_single_line.unwrap_or(true),
        }
    }
}

/// Preference of how list-like items are displayed.
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum ItemsLayout {
    /// Fit as much on one line as possible.
    Compressed,
    /// Items are placed horizontally if sufficient space, vertically otherwise.
    Tall,
    /// Place every item on a separate line.
    Vertical,
}

impl ItemsLayout {
    pub fn to_list_tactic(self, len: usize) -> ListTactic {
        match self {
            ItemsLayout::Compressed => ListTactic::Mixed,
            ItemsLayout::Tall => ListTactic::HorizontalVertical,
            ItemsLayout::Vertical if len == 1 => ListTactic::Horizontal,
            ItemsLayout::Vertical => ListTactic::Vertical,
        }
    }
}

/// Where to put the opening brace of items (`fn`, `impl`, etc.).
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum ItemBraceStyle {
    /// Put the opening brace on the next line.
    AlwaysNextLine,
    /// Put the opening brace on the same line, if possible.
    PreferSameLine,
    /// Prefer the same line except where there is a where-clause, in which
    /// case force the brace to be put on the next line.
    SameLineWhere,
}
