//! Configuration options related to formatting of expressions and punctuation.
use serde::{Deserialize, Serialize};

use super::items::ItemsLayout;

#[derive(Debug, Copy, Clone)]
pub struct Expressions {
    /////PUNCTUATION/////
    /// Brace style for control flow constructs.
    pub expr_brace_style: ExprBraceStyle,
    /// Add trailing semicolon after break, continue and return.
    pub trailing_semicolon: bool,
    /// Leave a space before the colon.
    pub space_before_colon: bool,
    /// Leave a space after the colon.
    pub space_after_colon: bool,

    /////OPERATORS/////
    /// Determines if `+` or `=` are wrapped in spaces in the punctuation of types.
    pub type_combinator_layout: TypeCombinatorLayout,
    /// Put spaces around the `..` and `..=` range operators.
    pub spaces_around_ranges: bool,

    /////MATCH EXPR/////
    /// Put a trailing comma after a block based match arm (non-block arms are not affected).
    pub match_block_trailing_comma: bool,
    /// Determines whether leading pipes are emitted on match arms.
    pub match_arm_leading_pipe: MatchArmLeadingPipe,

    /////FUNCTIONS/////
    /// Force multiline closure bodies and match arms to be wrapped in a block.
    pub force_multiline_blocks: bool,
    /// Control the layout of arguments in a function.
    pub fn_args_layout: ItemsLayout,
    /// Put single-expression functions on a single line.
    pub fn_single_line: bool,
}

impl Default for Expressions {
    fn default() -> Self {
        Self {
            expr_brace_style: ExprBraceStyle::AlwaysSameLine,
            trailing_semicolon: true,
            space_before_colon: false,
            space_after_colon: false,
            type_combinator_layout: TypeCombinatorLayout::Wide,
            spaces_around_ranges: false,
            match_block_trailing_comma: false,
            match_arm_leading_pipe: MatchArmLeadingPipe::Never,
            force_multiline_blocks: false,
            fn_args_layout: ItemsLayout::Tall,
            fn_single_line: false,
        }
    }
}

/////PUNCTUATION/////

/// Where to put the opening brace of conditional expressions (`if`, `match`, etc.).
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
