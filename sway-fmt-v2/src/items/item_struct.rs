use crate::{
    config::items::ItemBraceStyle,
    fmt::{Format, FormattedCode, Formatter},
    utils::{
        bracket::{AngleBracket, CurlyBrace},
        item_len::ItemLen,
    },
    FormatterError,
};
use std::fmt::Write;
use sway_parse::{token::Delimiter, token::PunctKind, ItemStruct};
use sway_types::Spanned;

impl Format for ItemStruct {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // Get the unformatted

        // Get struct_variant_align_threshold from config.
        let struct_variant_align_threshold =
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

        let multiline = !struct_lit_single_line || self.get_formatted_len() > struct_lit_width;
        format_struct(
            self,
            formatted_code,
            formatter,
            multiline,
            struct_variant_align_threshold,
        )?;
        Ok(())
    }
}

/// Format the struct if the multiline is passed as false struct will be formatted into a single line.
///
/// Example (multiline : false):
/// struct Foo { bar: u64,  baz: bool }
///
/// Example (multiline : true):
/// struct Foo {
///  bar: u64,
///  baz: bool,
/// }
fn format_struct(
    item_struct: &ItemStruct,
    formatted_code: &mut FormattedCode,
    formatter: &mut Formatter,
    multiline: bool,
    struct_variant_align_threshold: usize,
) -> Result<(), FormatterError> {
    // If there is a visibility token add it to the formatted_code with a ` ` after it.
    if let Some(visibility) = &item_struct.visibility {
        formatted_code.push_str(visibility.span().as_str());
        formatted_code.push(' ');
    }
    // Add struct token
    formatted_code.push_str(item_struct.struct_token.span().as_str());
    formatted_code.push(' ');

    // Add struct name
    formatted_code.push_str(item_struct.name.as_str());

    // Check if there is generic provided
    if let Some(generics) = &item_struct.generics {
        // Push open angle brace
        ItemStruct::open_angle_bracket(formatted_code, formatter)?;
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
        // Push closing angle brace
        ItemStruct::close_angle_bracket(formatted_code, formatter)?;
    }

    // Handle openning brace
    if multiline {
        ItemStruct::open_curly_brace(formatted_code, formatter)?;
        formatted_code.push('\n');
    } else {
        // Push a single whitespace before `{`
        formatted_code.push(' ');
        // Push open brace
        let open_brace = Delimiter::Brace.as_open_char();
        formatted_code.push(open_brace);
        // Push a single whitespace after `{`
        formatted_code.push(' ');
    }

    let items = item_struct
        .fields
        .clone()
        .into_inner()
        .value_separator_pairs;

    // In first iteration we are going to be collecting the lengths of the enum variants.
    let variant_length: Vec<usize> = items
        .iter()
        .map(|variant| variant.0.name.as_str().len())
        .collect();

    // Find the maximum length in the variant_length vector that is still smaller than enum_variant_align_threshold.
    let mut max_valid_variant_length = 0;

    variant_length.iter().for_each(|length| {
        if *length > max_valid_variant_length && *length < struct_variant_align_threshold {
            max_valid_variant_length = *length;
        }
    });
    for (item_index, item) in items.iter().enumerate() {
        if multiline {
            formatted_code.push_str(&formatter.shape.indent.to_string(formatter));
        }
        let type_field = &item.0;
        // Add name
        formatted_code.push_str(type_field.name.as_str());
        let current_variant_length = variant_length[item_index];
        if current_variant_length < max_valid_variant_length {
            // We need to add alignment between : and ty
            // max_valid_variant_length: the length of the variant that we are taking as a reference to allign
            // current_variant_length: the length of the current variant that we are trying to format
            let required_alignment = max_valid_variant_length - current_variant_length;
            // TODO: Improve handling this
            formatted_code.push_str(&(0..required_alignment).map(|_| ' ').collect::<String>());
        }
        // Add `:`
        formatted_code.push_str(type_field.colon_token.ident().as_str());
        formatted_code.push(' ');
        // TODO: We are currently converting ty to string directly but we will probably need to format ty before adding.
        // Add ty
        formatted_code.push_str(type_field.ty.span().as_str());
        // Add `, ` if this isn't the last field.
        if !multiline && item_index != items.len() - 1 {
            formatted_code.push_str(", ");
        } else if multiline {
            formatted_code.push_str(",\n");
        }
    }
    if !multiline {
        // Push a ' '
        formatted_code.push(' ');
    }
    // Handle closing brace
    ItemStruct::close_curly_brace(formatted_code, formatter)?;
    Ok(())
}

impl ItemLen for ItemStruct {
    fn get_formatted_len(&self) -> usize {
        // TODO while determininig the length we may want to format to some degree and take length.
        let str_item = &self.span().as_str().len();
        *str_item as usize
    }
}

impl CurlyBrace for ItemStruct {
    fn open_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let brace_style = formatter.config.items.item_brace_style;
        let extra_width = formatter.config.whitespace.tab_spaces;
        let open_brace = Delimiter::Brace.as_open_char();
        let mut shape = formatter.shape;
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add openning brace to the next line.
                write!(line, "\n{}", open_brace)?;
                shape = shape.block_indent(extra_width);
            }
            _ => {
                // Add opening brace to the same line
                write!(line, " {}", open_brace)?;
                shape = shape.block_indent(extra_width);
            }
        }

        formatter.shape = shape;
        Ok(())
    }

    fn close_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let close_brace = Delimiter::Brace.as_close_char();
        write!(line, "{}", close_brace)?;
        // If shape is becoming left-most alligned or - indent just have the defualt shape
        formatter.shape = formatter
            .shape
            .shrink_left(formatter.config.whitespace.tab_spaces)
            .unwrap_or_default();
        Ok(())
    }
}

impl AngleBracket for ItemStruct {
    fn open_angle_bracket(
        line: &mut String,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let open_angle = PunctKind::LessThan.as_char();
        write!(line, "{}", open_angle)?;
        Ok(())
    }

    fn close_angle_bracket(
        line: &mut String,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let close_angle = PunctKind::GreaterThan.as_char();
        write!(line, "{}", close_angle)?;
        Ok(())
    }
}
