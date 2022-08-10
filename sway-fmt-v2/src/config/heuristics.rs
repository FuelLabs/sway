//! Configuration options related to heuristics.
use crate::constants::{
    DEFAULT_ATTR_FN_LIKE_WIDTH, DEFAULT_CHAIN_WIDTH, DEFAULT_COLLECTION_WIDTH,
    DEFAULT_FN_CALL_WIDTH, DEFAULT_MAX_LINE_WIDTH, DEFAULT_SINGLE_LINE_IF_ELSE_WIDTH,
    DEFAULT_STRUCTURE_LIT_WIDTH, DEFAULT_STRUCTURE_VAR_WIDTH,
};
use serde::{Deserialize, Serialize};

use super::user_opts::HeuristicsOptions;

#[derive(Debug, Copy, Clone)]
pub struct Heuristics {
    /// Determines heuristics level of involvement.
    pub heuristics_pref: HeuristicsPreferences,
    /// Whether to use different formatting for items and expressions if they satisfy a heuristic notion of 'small'
    pub use_small_heuristics: bool,
}

impl Default for Heuristics {
    fn default() -> Self {
        Self {
            heuristics_pref: HeuristicsPreferences::Scaled,
            use_small_heuristics: true,
        }
    }
}

impl Heuristics {
    pub fn from_opts(opts: &HeuristicsOptions) -> Self {
        let default = Self::default();
        Self {
            heuristics_pref: opts.heuristics_pref.unwrap_or(default.heuristics_pref),
            use_small_heuristics: opts
                .use_small_heuristics
                .unwrap_or(default.use_small_heuristics),
        }
    }
}

/// Heuristic settings that can be used to simplify
/// the configuration of the granular width configurations
/// like `struct_lit_width`, `array_width`, etc.
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum HeuristicsPreferences {
    /// Turn off any heuristics
    Off,
    /// Turn on max heuristics
    Max,
    /// Use scaled values based on the value of `max_width`
    Scaled,
}

impl HeuristicsPreferences {
    pub fn to_width_heuristics(self, max_width: usize) -> WidthHeuristics {
        match self {
            HeuristicsPreferences::Off => WidthHeuristics::off(),
            HeuristicsPreferences::Max => WidthHeuristics::max(max_width),
            HeuristicsPreferences::Scaled => WidthHeuristics::scaled(max_width),
        }
    }
}

/// 'small' heuristic values
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Copy)]
pub struct WidthHeuristics {
    // Maximum width of the args of a function call before falling back
    // to vertical formatting.
    pub(crate) fn_call_width: usize,
    // Maximum width of the args of a function-like attributes before falling
    // back to vertical formatting.
    pub(crate) attr_fn_like_width: usize,
    // Maximum width in the body of a user-defined structure literal before falling back to
    // vertical formatting.
    pub(crate) structure_lit_width: usize,
    // Maximum width of a user-defined structure field before falling back
    // to vertical formatting.
    pub(crate) structure_field_width: usize,
    // Maximum width of a collection literal before falling back to vertical
    // formatting.
    pub(crate) collection_width: usize,
    // Maximum length of a chain to fit on a single line.
    pub(crate) chain_width: usize,
    // Maximum line length for single line if-else expressions. A value
    // of zero means always break if-else expressions.
    pub(crate) single_line_if_else_max_width: usize,
}

impl WidthHeuristics {
    /// Using this WidthHeuristics means we ignore heuristics.
    pub fn off() -> WidthHeuristics {
        WidthHeuristics {
            fn_call_width: usize::max_value(),
            attr_fn_like_width: usize::max_value(),
            structure_lit_width: 0,
            structure_field_width: 0,
            collection_width: usize::max_value(),
            chain_width: usize::max_value(),
            single_line_if_else_max_width: 0,
        }
    }

    pub fn max(max_width: usize) -> WidthHeuristics {
        WidthHeuristics {
            fn_call_width: max_width,
            attr_fn_like_width: max_width,
            structure_lit_width: max_width,
            structure_field_width: max_width,
            collection_width: max_width,
            chain_width: max_width,
            single_line_if_else_max_width: max_width,
        }
    }

    // scale the default WidthHeuristics according to max_width
    pub fn scaled(max_width: usize) -> WidthHeuristics {
        let max_width_ratio = if max_width > DEFAULT_MAX_LINE_WIDTH {
            let ratio = max_width as f32 / DEFAULT_MAX_LINE_WIDTH as f32;
            // round to the closest 0.1
            (ratio * 10.0).round() / 10.0
        } else {
            1.0
        };

        WidthHeuristics {
            fn_call_width: (DEFAULT_FN_CALL_WIDTH as f32 * max_width_ratio).round() as usize,
            attr_fn_like_width: (DEFAULT_ATTR_FN_LIKE_WIDTH as f32 * max_width_ratio).round()
                as usize,
            structure_lit_width: (DEFAULT_STRUCTURE_LIT_WIDTH as f32 * max_width_ratio).round()
                as usize,
            structure_field_width: (DEFAULT_STRUCTURE_VAR_WIDTH as f32 * max_width_ratio).round()
                as usize,
            collection_width: (DEFAULT_COLLECTION_WIDTH as f32 * max_width_ratio).round() as usize,
            chain_width: (DEFAULT_CHAIN_WIDTH as f32 * max_width_ratio).round() as usize,
            single_line_if_else_max_width: (DEFAULT_SINGLE_LINE_IF_ELSE_WIDTH as f32
                * max_width_ratio)
                .round() as usize,
        }
    }
}

impl Default for WidthHeuristics {
    fn default() -> Self {
        Self::scaled(100)
    }
}
