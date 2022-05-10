use anyhow::{anyhow, Result};
use forc_util::{find_parent_dir_with_file, println_yellow_err};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub use crate::error::FormatterError;
use crate::{
    config::{
        comments::Comments, expr::Expressions, heuristics::Heuristics, imports::Imports,
        items::Items, lists::Lists, literals::Literals, ordering::Ordering, user_def::Structures,
        user_set::*, whitespace::Whitespace,
    },
    constants::SWAY_FORMAT_FILE_NAME,
};

/// A finalized `swayfmt` config.
#[derive(Debug)]
pub struct FormatConfig {
    pub whitespace: Whitespace,
    pub imports: Imports,
    pub ordering: Ordering,
    pub items: Items,
    pub lists: Lists,
    pub literals: Literals,
    pub expressions: Expressions,
    pub heuristics: Heuristics,
    pub structures: Structures,
    pub comments: Comments,
}

/// A direct mapping to an optional `swayfmt.toml`.
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub struct OptionFormatConfig {
    pub whitespace: Option<WhitespaceOptions>,
    pub imports: Option<ImportsOptions>,
    pub ordering: Option<OrderingOptions>,
    pub items: Option<ItemsOptions>,
    pub lists: Option<ListsOptions>,
    pub literals: Option<LiteralsOptions>,
    pub expressions: Option<ExpressionsOptions>,
    pub heuristics: Option<HeuristicsOptions>,
    pub structures: Option<StructuresOptions>,
    pub comments: Option<CommentsOptions>,
}

impl FormatConfig {
    /// The default setting of `swayfmt`'s `FormatConfig`.
    ///
    pub fn default() -> Self {
        Self {
            whitespace: Whitespace::default(),
            imports: Imports::default(),
            ordering: Ordering::default(),
            items: Items::default(),
            lists: Lists::default(),
            literals: Literals::default(),
            expressions: Expressions::default(),
            heuristics: Heuristics::default(),
            structures: Structures::default(),
            comments: Comments::default(),
        }
    }
    /// Given an optional path to a `swayfmt.toml`, read it and construct a `FormatConfig`.
    /// If settings are omitted, those fields will be set to default. If `None` is provided,
    /// the default config will be applied. If a `swayfmt.toml` exists but is empty, the default
    /// config will be applied.
    ///
    /// At present, this will only return a warning if it catches unusable fields.
    /// Upon completion, this should give errors/warnings for incorrect input fields as well.
    ///
    pub fn from_dir_or_default(config_path: Option<&Path>) -> Result<Self> {
        let config = OptionFormatConfig::from_dir_or_default(config_path)?;
        Ok(config)
    }
}

impl OptionFormatConfig {
    /// Given an optional path to a `swayfmt.toml`, read it and construct a `FormatConfig`.
    /// If settings are omitted, those fields will be set to default. If `None` is provided,
    /// the default config will be applied. If a `swayfmt.toml` exists but is empty, the default
    /// config will be applied.
    ///
    /// At present, this will only return a warning if it catches unusable fields.
    /// Upon completion, this should give errors/warnings for incorrect input fields as well.
    ///
    pub fn from_dir_or_default(config_path: Option<&Path>) -> Result<FormatConfig> {
        match config_path {
            Some(starter_path) => {
                if let Some(path) = find_parent_dir_with_file(starter_path, SWAY_FORMAT_FILE_NAME) {
                    let config_str = std::fs::read_to_string(path)
                        .map_err(|e| anyhow!("failed to read config at {:?}: {}", path, e))?;
                    // save some time if the file is empty
                    if !config_str.is_empty() {
                        let toml_de = &mut toml::de::Deserializer::new(&config_str);
                        let user_settings: Self = serde_ignored::deserialize(toml_de, |field| {
                            let warning =
                                format!("  WARNING! found unusable configuration: {}", field);
                            println_yellow_err(&warning);
                        })
                        .map_err(|e| anyhow!("failed to parse config: {}.", e))?;

                        Ok(Self::apply_user_settings(user_settings))
                    } else {
                        Ok(FormatConfig::default())
                    }
                } else {
                    Ok(FormatConfig::default())
                }
            }
            None => Ok(FormatConfig::default()),
        }
    }
    /// Check the user's settings, and replace the values of the default formatter
    /// with those of the user's if they exist.
    ///
    pub fn apply_user_settings(user_settings: OptionFormatConfig) -> FormatConfig {
        let config: FormatConfig = FormatConfig::default();

        if let Some(whitespace) = user_settings.whitespace {
            if let Some(max_width) = whitespace.max_width {
                config.whitespace.max_width = max_width;
            }
            if let Some(hard_tabs) = whitespace.hard_tabs {
                config.whitespace.hard_tabs = hard_tabs;
            }
            if let Some(tab_spaces) = whitespace.tab_spaces {
                config.whitespace.tab_spaces = tab_spaces;
            }
            if let Some(newline_style) = whitespace.newline_style {
                config.whitespace.newline_style = newline_style;
            }
            if let Some(indent_style) = whitespace.indent_style {
                config.whitespace.indent_style = indent_style;
            }
        }
        if let Some(imports) = user_settings.imports {
            if let Some(group_imports) = imports.group_imports {
                config.imports.group_imports = group_imports;
            }
            if let Some(granularity) = imports.imports_granularity {
                config.imports.imports_granularity = granularity;
            }
            if let Some(indent) = imports.imports_indent {
                config.imports.imports_indent = indent;
            }
            if let Some(layout) = imports.imports_layout {
                config.imports.imports_layout = layout;
            }
        }
        if let Some(ordering) = user_settings.ordering {
            if let Some(reorder_imports) = ordering.reorder_imports {
                config.ordering.reorder_imports = reorder_imports
            }
            if let Some(reorder_modules) = ordering.reorder_modules {
                config.ordering.reorder_modules = reorder_modules
            }
            if let Some(reorder_impl_items) = ordering.reorder_impl_items {
                config.ordering.reorder_impl_items = reorder_impl_items
            }
        }
        if let Some(items) = user_settings.items {
            if let Some(brace_style) = items.item_brace_style {
                config.items.item_brace_style = brace_style;
            }
            if let Some(upper_bound) = items.blank_lines_upper_bound {
                config.items.blank_lines_upper_bound = upper_bound;
            }
            if let Some(lower_bound) = items.blank_lines_lower_bound {
                config.items.blank_lines_lower_bound = lower_bound;
            }
            if let Some(single_line) = items.empty_item_single_line {
                config.items.empty_item_single_line = single_line;
            }
        }
        if let Some(lists) = user_settings.lists {
            if let Some(trailing_comma) = lists.trailing_comma {
                config.lists.trailing_comma = trailing_comma;
            }
        }
        if let Some(literals) = user_settings.literals {
            if let Some(format_strings) = literals.format_strings {
                config.literals.format_strings = format_strings;
            }
            if let Some(hex_case) = literals.hex_literal_case {
                config.literals.hex_literal_case = hex_case;
            }
        }
        if let Some(expressions) = user_settings.expressions {
            if let Some(brace_style) = expressions.expr_brace_style {
                config.expressions.expr_brace_style = brace_style;
            }
            if let Some(trailing_semicolon) = expressions.trailing_semicolon {
                config.expressions.trailing_semicolon = trailing_semicolon;
            }
            if let Some(space_before_colon) = expressions.space_before_colon {
                config.expressions.space_before_colon = space_before_colon;
            }
            if let Some(space_after_colon) = expressions.space_after_colon {
                config.expressions.space_after_colon = space_after_colon;
            }
            if let Some(type_comb_layout) = expressions.type_combinator_layout {
                config.expressions.type_combinator_layout = type_comb_layout;
            }
            if let Some(spaces_around_ranges) = expressions.spaces_around_ranges {
                config.expressions.spaces_around_ranges = spaces_around_ranges;
            }
            if let Some(match_block_trailing_comma) = expressions.match_block_trailing_comma {
                config.expressions.match_block_trailing_comma = match_block_trailing_comma;
            }
            if let Some(match_arm_leading_pipe) = expressions.match_arm_leading_pipe {
                config.expressions.match_arm_leading_pipe = match_arm_leading_pipe;
            }
            if let Some(multiline_blocks) = expressions.force_multiline_blocks {
                config.expressions.force_multiline_blocks = multiline_blocks;
            }
            if let Some(fn_args_layout) = expressions.fn_args_layout {
                config.expressions.fn_args_layout = fn_args_layout;
            }
            if let Some(fn_single_line) = expressions.fn_single_line {
                config.expressions.fn_single_line = fn_single_line;
            }
        }
        if let Some(heuristics) = user_settings.heuristics {
            if let Some(heuristics) = heuristics.heuristics_pref {
                config.heuristics.heuristics_pref = heuristics;
            }
            if let Some(width) = heuristics.width_heuristics {
                config.heuristics.width_heuristics = width;
            }
            if let Some(small_heuristics) = heuristics.use_small_heuristics {
                config.heuristics.use_small_heuristics = small_heuristics;
            }
        }
        if let Some(structures) = user_settings.structures {
            if let Some(variant_threshold) = structures.enum_variant_align_threshold {
                config.structures.enum_variant_align_threshold = variant_threshold;
            }
            if let Some(field_threshold) = structures.struct_field_align_threshold {
                config.structures.struct_field_align_threshold = field_threshold;
            }
            if let Some(struct_lit_single_line) = structures.struct_lit_single_line {
                config.structures.struct_lit_single_line = struct_lit_single_line;
            }
        }
        if let Some(comments) = user_settings.comments {
            if let Some(wrap_comments) = comments.wrap_comments {
                config.comments.wrap_comments = wrap_comments;
            }
            if let Some(width) = comments.comment_width {
                config.comments.comment_width = width;
            }
            if let Some(normalize) = comments.normalize_comments {
                config.comments.normalize_comments = normalize;
            }
        }

        config
    }
}
