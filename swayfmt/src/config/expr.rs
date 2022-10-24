//! Configuration options related to formatting of expressions and punctuation.
use crate::config::{items::ItemsLayout, user_opts::ExpressionsOptions};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone)]
pub struct Expressions {
    /////PUNCTUATION/////
    /// Brace style for control flow constructs.
    pub expr_brace_style:   ExprBraceStyle,
    /// Add trailing semicolon after break, continue and return.
    pub trailing_semicolon: bool,
    /// Leave a space before the colon.
    pub space_before_colon: bool,
    /// Leave a space after the colon.
    pub space_after_colon:  bool,

    /////OPERATORS/////
    /// Determines if `+` or `=` are wrapped in spaces in the punctuation of types.
    pub type_combinator_layout: TypeCombinatorLayout,
    /// Put spaces around the `..` and `..=` range operators.
    pub spaces_around_ranges:   bool,

    /////MATCH EXPR/////
    /// Put a trailing comma after a block based match arm (non-block arms are not affected).
    pub match_block_trailing_comma: bool,
    /// Determines whether leading pipes are emitted on match arms.
    pub match_arm_leading_pipe:     MatchArmLeadingPipe,

    /////FUNCTIONS/////
    /// Force multiline closure bodies and match arms to be wrapped in a block.
    pub force_multiline_blocks: bool,
    /// Control the layout of arguments in a function.
    pub fn_args_layout:         ItemsLayout,
    /// Put single-expression functions on a single line.
    pub fn_single_line:         bool,
}

impl Default for Expressions {
    fn default() -> Self {
        Self {
            expr_brace_style:           ExprBraceStyle::AlwaysSameLine,
            trailing_semicolon:         true,
            space_before_colon:         false,
            space_after_colon:          false,
            type_combinator_layout:     TypeCombinatorLayout::Wide,
            spaces_around_ranges:       false,
            match_block_trailing_comma: false,
            match_arm_leading_pipe:     MatchArmLeadingPipe::Never,
            force_multiline_blocks:     false,
            fn_args_layout:             ItemsLayout::Tall,
            fn_single_line:             false,
        }
    }
}

impl Expressions {
    pub fn from_opts(opts: &ExpressionsOptions) -> Self {
        let default = Self::default();
        Self {
            expr_brace_style:           opts.expr_brace_style.unwrap_or(default.expr_brace_style),
            trailing_semicolon:         opts
                .trailing_semicolon
                .unwrap_or(default.trailing_semicolon),
            space_before_colon:         opts
                .space_before_colon
                .unwrap_or(default.space_before_colon),
            space_after_colon:          opts.space_after_colon.unwrap_or(default.space_after_colon),
            type_combinator_layout:     opts
                .type_combinator_layout
                .unwrap_or(default.type_combinator_layout),
            spaces_around_ranges:       opts
                .spaces_around_ranges
                .unwrap_or(default.spaces_around_ranges),
            match_block_trailing_comma: opts
                .match_block_trailing_comma
                .unwrap_or(default.match_block_trailing_comma),
            match_arm_leading_pipe:     opts
                .match_arm_leading_pipe
                .unwrap_or(default.match_arm_leading_pipe),
            force_multiline_blocks:     opts
                .force_multiline_blocks
                .unwrap_or(default.force_multiline_blocks),
            fn_args_layout:             opts.fn_args_layout.unwrap_or(default.fn_args_layout),
            fn_single_line:             opts.fn_single_line.unwrap_or(default.fn_single_line),
        }
    }
}

/////PUNCTUATION/////

/// Where to put the opening brace of conditional expressions (`if`, `match`, etc.).
#[allow(clippy::enum_variant_names)]
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum ExprBraceStyle {
    /// K&R style, Rust community default
    AlwaysSameLine,
    /// Stroustrup style
    ClosingNextLine,
    /// Allman style
    AlwaysNextLine,
}

/// Spacing around type combinators.
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum TypeCombinatorLayout {
    /// No spaces around "=" and "+"
    Compressed,
    /// Spaces around " = " and " + "
    Wide,
}

/////MATCH EXPR/////

/// Controls how swayfmt should handle leading pipes on match arms.
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum MatchArmLeadingPipe {
    /// Place leading pipes on all match arms
    Always,
    /// Never emit leading pipes on match arms
    Never,
    /// Preserve any existing leading pipes
    Preserve,
}
