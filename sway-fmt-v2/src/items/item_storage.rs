use crate::{
    config::{items::ItemBraceStyle, user_def::FieldAlignment},
    fmt::{Format, FormattedCode, Formatter},
    utils::{bracket::CurlyBrace, item_len::ItemLen},
    FormatterError,
};
use std::fmt::Write;
use sway_parse::{
    token::{Delimiter, PunctKind},
    ItemStorage,
};
use sway_types::Spanned;

impl Format for ItemStorage {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // Should we format small storage into single line
        let storage_single_line = formatter.config.structures.small_structures_single_line;

        // Get the width limit of a storage to be formatted into single line if storage_single_line is true
        let config_whitespace = formatter.config.whitespace;
        let width_heuristics = formatter
            .config
            .heuristics
            .heuristics_pref
            .to_width_heuristics(&config_whitespace);
        let storage_width = width_heuristics.structure_lit_width;

        let multiline = !storage_single_line || self.get_formatted_len()? > storage_width;
        format_storage(self, formatter, formatted_code, multiline)?;
        Ok(())
    }
}

impl ItemLen for ItemStorage {
    fn get_formatted_len(&self) -> Result<usize, FormatterError> {
        // Format to single line and return the length
        let mut str_item = String::new();
        let mut formatter = Formatter::default();
        format_storage(self, &mut formatter, &mut str_item, false)?;
        Ok(str_item.len() as usize)
    }
}

fn format_storage(
    item_storage: &ItemStorage,
    formatter: &mut Formatter,
    formatted_code: &mut String,
    multiline: bool,
) -> Result<(), FormatterError> {
    // Add storage token
    formatted_code.push_str(item_storage.storage_token.span().as_str());
    let fields = item_storage.fields.clone().into_inner();

    // Handle openning brace
    ItemStorage::open_curly_brace(formatted_code, formatter)?;
    if multiline {
        formatted_code.push('\n');
        // Determine alignment tactic
        match formatter.config.structures.field_alignment {
            FieldAlignment::AlignFields(storage_field_align_threshold) => {
                let value_pairs = fields.value_separator_pairs;
                // In first iteration we are going to be collecting the lengths of the struct fields.
                let field_length: Vec<usize> = value_pairs
                    .iter()
                    .map(|field| field.0.name.as_str().len())
                    .collect();

                // Find the maximum length in the `field_length` vector that is still smaller than `storage_field_align_threshold`.
                // `max_valid_field_length`: the length of the field that we are taking as a reference to align.
                let mut max_valid_field_length = 0;
                field_length.iter().for_each(|length| {
                    if *length > max_valid_field_length && *length < storage_field_align_threshold {
                        max_valid_field_length = *length;
                    }
                });

                let mut value_pairs_iter = value_pairs.iter().enumerate().peekable();
                for (field_index, field) in value_pairs_iter.clone() {
                    formatted_code.push_str(&formatter.shape.indent.to_string(formatter));

                    let type_field = &field.0;
                    // Add name
                    formatted_code.push_str(type_field.name.as_str());

                    // `current_field_length`: the length of the current field that we are trying to format.
                    let current_field_length = field_length[field_index];
                    if current_field_length < max_valid_field_length {
                        // We need to add alignment between `:` and `ty`
                        let mut required_alignment = max_valid_field_length - current_field_length;
                        while required_alignment != 0 {
                            formatted_code.push(' ');
                            required_alignment -= 1;
                        }
                    }
                    // Add `:`, `ty` & `CommaToken`
                    //
                    // TODO(#2101): We are currently converting ty to string directly but
                    // we will probably need to format `ty` before adding.
                    write!(
                        formatted_code,
                        "{} {}",
                        type_field.colon_token.ident().as_str(),
                        type_field.ty.span().as_str(),
                    )?;
                    if value_pairs_iter.peek().is_some() {
                        writeln!(formatted_code, "{}", field.1.span().as_str())?;
                    } else if let Some(final_value) = &fields.final_value_opt {
                        final_value.format(formatted_code, formatter)?;
                    }
                }
            }
            FieldAlignment::Off => {
                let mut value_pairs_iter = fields.value_separator_pairs.iter().peekable();
                for field in value_pairs_iter.clone() {
                    formatted_code.push_str(&formatter.shape.indent.to_string(formatter));
                    let item_field = &field.0;
                    item_field.format(formatted_code, formatter)?;

                    if value_pairs_iter.peek().is_some() {
                        writeln!(formatted_code, "{}", field.1.span().as_str())?;
                    }
                }
                if let Some(final_value) = &fields.final_value_opt {
                    formatted_code.push_str(&formatter.shape.indent.to_string(formatter));
                    final_value.format(formatted_code, formatter)?;
                    writeln!(formatted_code, "{}", PunctKind::Comma.as_char())?;
                }
            }
        }
    } else {
        // non-multiline formatting
        formatted_code.push(' ');
        let mut value_pairs_iter = fields.value_separator_pairs.iter().peekable();
        for field in value_pairs_iter.clone() {
            let type_field = &field.0;
            type_field.format(formatted_code, formatter)?;

            if value_pairs_iter.peek().is_some() {
                write!(formatted_code, "{} ", field.1.span().as_str())?;
            }
        }
        if let Some(final_value) = &fields.final_value_opt {
            final_value.format(formatted_code, formatter)?;
            formatted_code.push(' ');
        } else {
            formatted_code.pop();
            formatted_code.pop();
            formatted_code.push(' ');
        }
    }

    // Handle closing brace
    ItemStorage::close_curly_brace(formatted_code, formatter)?;
    Ok(())
}

impl CurlyBrace for ItemStorage {
    fn open_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let brace_style = formatter.config.items.item_brace_style;
        let extra_width = formatter.config.whitespace.tab_spaces;
        let mut shape = formatter.shape;
        let open_brace = Delimiter::Brace.as_open_char();
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add opening brace to the next line.
                line.push_str(&format!("\n{}", open_brace));
                shape = shape.block_indent(extra_width);
            }
            _ => {
                // Add opening brace to the same line
                line.push_str(&format!(" {}", open_brace));
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
        line.push(Delimiter::Brace.as_close_char());
        // shrink_left would return error if the current indentation level is becoming < 0, in that
        // case we should use the Shape::default() which has 0 indentation level.
        formatter.shape = formatter
            .shape
            .shrink_left(formatter.config.whitespace.tab_spaces)
            .unwrap_or_default();
        Ok(())
    }
}
