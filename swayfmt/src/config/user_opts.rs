//! All of the user-facing configuration options stored in [ConfigOptions].
use crate::config::{
    expr::{ExprBraceStyle, MatchArmLeadingPipe, TypeCombinatorLayout},
    heuristics::HeuristicsPreferences,
    imports::{GroupImports, ImportGranularity},
    items::{ItemBraceStyle, ItemsLayout},
    lists::{ListTactic, SeparatorTactic},
    literals::HexLiteralCase,
    user_def::FieldAlignment,
    whitespace::{IndentStyle, NewlineStyle},
};
use serde::{Deserialize, Serialize};
/// See parent struct [Whitespace].
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct WhitespaceOptions {
    pub max_width:         Option<usize>,
    pub hard_tabs:         Option<bool>,
    pub tab_spaces:        Option<usize>,
    pub newline_style:     Option<NewlineStyle>,
    pub indent_style:      Option<IndentStyle>,
    pub newline_threshold: Option<usize>,
}
/// See parent struct [Imports].
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct ImportsOptions {
    pub group_imports:       Option<GroupImports>,
    pub imports_granularity: Option<ImportGranularity>,
    pub imports_indent:      Option<IndentStyle>,
    pub imports_layout:      Option<ListTactic>,
}
/// See parent struct [Ordering].
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct OrderingOptions {
    pub reorder_imports:    Option<bool>,
    pub reorder_modules:    Option<bool>,
    pub reorder_impl_items: Option<bool>,
}
/// See parent struct [Items].
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct ItemsOptions {
    pub item_brace_style:        Option<ItemBraceStyle>,
    pub blank_lines_upper_bound: Option<usize>,
    pub blank_lines_lower_bound: Option<usize>,
    pub empty_item_single_line:  Option<bool>,
}
/// See parent struct [Lists].
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct ListsOptions {
    pub trailing_comma: Option<SeparatorTactic>,
}
/// See parent struct [Literals].
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct LiteralsOptions {
    pub format_strings:   Option<bool>,
    pub hex_literal_case: Option<HexLiteralCase>,
}
/// See parent struct [Expressions].
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct ExpressionsOptions {
    pub expr_brace_style:           Option<ExprBraceStyle>,
    pub trailing_semicolon:         Option<bool>,
    pub space_before_colon:         Option<bool>,
    pub space_after_colon:          Option<bool>,
    pub type_combinator_layout:     Option<TypeCombinatorLayout>,
    pub spaces_around_ranges:       Option<bool>,
    pub match_block_trailing_comma: Option<bool>,
    pub match_arm_leading_pipe:     Option<MatchArmLeadingPipe>,
    pub force_multiline_blocks:     Option<bool>,
    pub fn_args_layout:             Option<ItemsLayout>,
    pub fn_single_line:             Option<bool>,
}
/// See parent struct [Heuristics].
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct HeuristicsOptions {
    pub heuristics_pref:      Option<HeuristicsPreferences>,
    pub use_small_heuristics: Option<bool>,
}
/// See parent struct [Structures].
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct StructuresOptions {
    pub field_alignment:        Option<FieldAlignment>,
    pub struct_lit_single_line: Option<bool>,
}
/// See parent struct [Comments].
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct CommentsOptions {
    pub wrap_comments:      Option<bool>,
    pub comment_width:      Option<usize>,
    pub normalize_comments: Option<bool>,
}
