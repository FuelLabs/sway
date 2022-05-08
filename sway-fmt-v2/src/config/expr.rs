use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Expressions {
    /// Brace style for control flow constructs.
    pub expr_brace_style: ExprBraceStyle,
    /// Add trailing semicolon after break, continue and return.
    pub trailing_semicolon: bool,
    /// Leave a space before the colon.
    pub space_before_colon: bool,
    /// Leave a space after the colon.
    pub space_after_colon: bool,
    /// Determines if `+` or `=` are wrapped in spaces in the punctuation of types.
    pub type_punctuation_layout: TypeCombinatorLayout,
    /// Put spaces around the `..` and `..=` range operators.
    pub spaces_around_ranges: bool,
    /// Put a trailing comma after a block based match arm (non-block arms are not affected).
    pub match_block_trailing_comma: bool,
    /// Determines whether leading pipes are emitted on match arms.
    pub match_arm_leading_pipe: MatchArmLeadingPipe,
}

/////PUNCTUATION/////

/// Where to put the opening brace of conditional expressions (`if`, `match`, etc.).
#[derive(Serialize, Deserialize, Debug)]
pub enum ExprBraceStyle {
    /// K&R style, Rust community default
    AlwaysSameLine,
    /// Stroustrup style
    ClosingNextLine,
    /// Allman style
    AlwaysNextLine,
}

/// Spacing around type combinators.
#[derive(Serialize, Deserialize, Debug)]
pub enum TypeCombinatorLayout {
    /// No spaces around "=" and "+"
    Compressed,
    /// Spaces around " = " and " + "
    Wide,
}

/////MATCH EXPR/////

/// Controls how swayfmt should handle leading pipes on match arms.
#[derive(Serialize, Deserialize, Debug)]
pub enum MatchArmLeadingPipe {
    /// Place leading pipes on all match arms
    Always,
    /// Never emit leading pipes on match arms
    Never,
    /// Preserve any existing leading pipes
    Preserve,
}
