use crate::config::lists::ListTactic;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Items {
    pub items_layout: ItemsLayout,
    pub item_brace_style: ItemBraceStyle,
    /// Maximum number of blank lines which can be put between items.
    pub blank_lines_upper_bound: usize,
    /// Minimum number of blank lines which must be put between items.
    pub blank_lines_lower_bound: usize,
}

/// Preference of how items are displayed.
#[derive(Serialize, Deserialize, Debug)]
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
#[derive(Serialize, Deserialize, Debug)]
pub enum ItemBraceStyle {
    /// Put the opening brace on the next line.
    AlwaysNextLine,
    /// Put the opening brace on the same line, if possible.
    PreferSameLine,
    /// Prefer the same line except where there is a where-clause, in which
    /// case force the brace to be put on the next line.
    SameLineWhere,
}
