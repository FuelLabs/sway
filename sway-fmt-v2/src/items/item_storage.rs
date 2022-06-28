use crate::{
    config::items::ItemBraceStyle,
    fmt::{Format, FormattedCode, Formatter},
    utils::{bracket::CurlyBrace, item_len::ItemLen},
    FormatterError,
};
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
        let storage_single_line = formatter.config.structures.storage_single_line;

        // Get the width limit of a storage to be formatted into single line if storage_single_line is true
        let config_whitespace = formatter.config.whitespace;
        let width_heuristics = formatter
            .config
            .heuristics
            .heuristics_pref
            .to_width_heuristics(&config_whitespace);
        let storage_width = width_heuristics.storage_width;

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
    // Get storage field alignment threshold
    let storage_field_align_threshold = formatter.config.structures.storage_field_align_threshold;
    // Add storage token
    formatted_code.push_str(item_storage.storage_token.span().as_str());

    // Add `{`
    ItemStorage::open_curly_brace(formatted_code, formatter)?;
    let offset = if multiline { '\n' } else { ' ' };
    formatted_code.push(offset);
    // Get the fields
    let items = item_storage.fields.clone().into_inner();

    // In first iteration we are going to be collecting the lengths of the enum variants.
    let variant_length: Vec<usize> = items
        .clone()
        .into_iter()
        .map(|variant| variant.name.as_str().len())
        .collect();
    // Find the maximum length in the variant_length vector that is still smaller than enum_variant_align threshold.
    let mut max_valid_variant_length = 0;

    variant_length.iter().for_each(|length| {
        if *length > max_valid_variant_length && *length < storage_field_align_threshold {
            max_valid_variant_length = *length;
        }
    });

    for (item_index, item) in items.into_iter().enumerate() {
        if multiline {
            // If we are formatting to multiline push the current indentation level
            formatted_code.push_str(&formatter.shape.indent.to_string(formatter));
        }
        // Push the storage field name
        formatted_code.push_str(item.name.as_str());

        // Alignment only makes sense when we are formatting to multiline
        if multiline {
            let current_variant_length = variant_length[item_index];
            if current_variant_length < max_valid_variant_length {
                // We need to add alignment between : and ty
                // max_valid_variant_length: the length of the variant that we are taking as a reference to align
                // current_variant_length: the length of the current variant that we are trying to format
                let required_alignment = max_valid_variant_length - current_variant_length;
                formatted_code.push_str(&(0..required_alignment).map(|_| ' ').collect::<String>());
            }
        }

        // Push the colon token
        formatted_code.push_str(item.colon_token.span().as_str());
        formatted_code.push(' ');

        // Push the ty
        formatted_code.push_str(item.ty.span().as_str());

        // Push initializer if it exists.
        if let Some(initializer) = item.initializer {
            // Push a ` `
            formatted_code.push(' ');

            let expr = initializer.1;

            // Push the `=`
            formatted_code.push(PunctKind::Equals.as_char());

            // Push a ` `
            formatted_code.push(' ');

            // Push the unformatted expr
            formatted_code.push_str(expr.span().as_str());
        }

        formatted_code.push(PunctKind::Comma.as_char());
        // If we are formatting to multiple lines push a `\n` before the next item
        // For single line add a single space.
        formatted_code.push(offset);
    }
    // Add `}`
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
