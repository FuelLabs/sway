use crate::{
    config::items::ItemBraceStyle,
    fmt::{Format, FormattedCode, Formatter},
    utils::{
        bracket::{AngleBracket, CurlyBrace},
        item_len::ItemLen,
    },
};
use sway_parse::ItemStruct;
use sway_types::Spanned;

impl Format for ItemStruct {
    fn format(&self, formatter: &mut Formatter) -> FormattedCode {
        // TODO: creating this formatted_code with FormattedCode::new() will likely cause lots of
        // reallocations maybe we can explore how we can do this, starting with with_capacity may help.
        let mut formatted_code = FormattedCode::new();

        // Get the unformatted

        // Get struct_variant_align_threshold from config.
        let _struct_variant_align_threshold =
            formatter.config.structures.struct_field_align_threshold;

        // Should small structs formatted into a single line.
        let struct_lit_single_line = formatter.config.structures.struct_lit_single_line;

        // Get the width limit of a struct to be formatted into single line if struct_lit_single_line is true.
        let config_whitespace = formatter.config.whitespace;
        let width_heuristics = formatter
            .config
            .heuristics
            .heuristics_pref
            .to_width_heuristics(&config_whitespace);
        let struct_lit_width = width_heuristics.struct_lit_width;

        if struct_lit_single_line {
            // Check if the struct len is smaller than struct_lit_width
            if self.get_formatted_len() < struct_lit_width {
                handle_struct_lit_single_line(self, &mut formatted_code, formatter);
            }
        }

        formatted_code
    }
}

/// Handles the scenerio when struct literals should be formatted to a single line.
fn handle_struct_lit_single_line(
    item_struct: &ItemStruct,
    formatted_code: &mut FormattedCode,
    formatter: &mut Formatter,
) {
    // If there is a visibility token add it to the formatted_code with a ` ` after it.
    if let Some(visibility) = &item_struct.visibility {
        formatted_code.push_str(visibility.span().as_str());
        formatted_code.push(' ');
    }
    // Add struct name
    formatted_code.push_str(item_struct.name.as_str());

    // Check if there is generic provided
    if let Some(generics) = &item_struct.generics {
        // Push angle brace
        ItemStruct::open_angle_bracket(formatted_code, formatter);
        // Get generics fields
        let generics = generics.parameters.inner.value_separator_pairs.clone();
        for (index, generic) in generics.iter().enumerate() {
            // Push ident
            formatted_code.push_str(generic.0.as_str());
            if index != generics.len() - 1 {
                // Push `, ` if this is not the last generic
                formatted_code.push_str(", ");
            }
        }
    }
    // Handle openning brace
    ItemStruct::open_curly_brace(formatted_code, formatter);

    // Push the current indentation level after `{`
    formatted_code.push_str(&formatter.shape.indent.to_string(formatter));

    let items = item_struct
        .fields
        .clone()
        .into_inner()
        .value_separator_pairs;
    for (item_index, item) in items.iter().enumerate() {
        let type_field = &item.0;
        // Add name
        formatted_code.push_str(type_field.name.as_str());
        // Add `:`
        formatted_code.push_str(type_field.colon_token.ident().as_str());
        // TODO: We are currently converting ty to string directly but we will probably need to format ty before adding.
        // Add ty
        formatted_code.push_str(type_field.ty.span().as_str());
        // Add `, ` if this isn't the last field.
        if item_index != items.len() - 1 {
            formatted_code.push_str(", ");
        }
    }
    // Push a ' '
    formatted_code.push(' ');

    // Handle closing brace
    ItemStruct::close_curly_brace(formatted_code, formatter);
}

impl ItemLen for ItemStruct {
    fn get_formatted_len(&self) -> usize {
        // TODO while determininig the length we may want to format to some degree and take length.
        let str_item = &self.span().as_str().len();
        *str_item as usize
    }
}

impl CurlyBrace for ItemStruct {
    fn open_curly_brace(line: &mut String, formatter: &mut Formatter) {
        let brace_style = formatter.config.items.item_brace_style;
        let mut shape = formatter.shape;
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add openning brace to the next line.
                line.push_str("\n{");
                shape = shape.block_indent(1);
            }
            _ => {
                // Add opening brace to the same line
                line.push_str(" {");
                shape = shape.block_indent(1);
            }
        }

        formatter.shape = shape;
    }

    fn close_curly_brace(line: &mut String, formatter: &mut Formatter) {
        line.push('}');
        // If shape is becoming left-most alligned or - indent just have the defualt shape
        formatter.shape = formatter.shape.shrink_left(1).unwrap_or_default();
    }
}

impl AngleBracket for ItemStruct {
    fn open_angle_bracket(line: &mut String, _formatter: &mut Formatter) {
        line.push('<');
    }

    fn close_angle_bracket(line: &mut String, _formatter: &mut Formatter) {
        line.push('>');
    }
}
