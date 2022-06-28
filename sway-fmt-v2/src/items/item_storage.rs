use crate::{
    config::items::ItemBraceStyle,
    fmt::{Format, FormattedCode, Formatter},
    utils::bracket::CurlyBrace,
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
        // Get storage field alignment threshold
        let storage_field_align_threshold =
            formatter.config.structures.storage_field_align_threshold;
        // Add storage token
        formatted_code.push_str(self.storage_token.span().as_str());

        // Add `{`
        Self::open_curly_brace(formatted_code, formatter)?;

        // Get the fields
        let items = self.fields.clone().into_inner();

        // In first iteration we are going to be collecting the lengths of the enum variants.
        let variant_length: Vec<usize> = items
            .clone()
            .into_iter()
            .map(|variant| variant.name.as_str().len())
            .collect();

        // Find the maximum length in the variant_length vector that is still smaller than enum_variant_align_threshold.
        let mut max_valid_variant_length = 0;

        variant_length.iter().for_each(|length| {
            if *length > max_valid_variant_length && *length < storage_field_align_threshold {
                max_valid_variant_length = *length;
            }
        });

        for (item_index, item) in items.into_iter().enumerate() {
            // Push the current indentation level
            formatted_code.push_str(&formatter.shape.indent.to_string(formatter));

            // Push the storage field name
            formatted_code.push_str(item.name.as_str());

            let current_variant_length = variant_length[item_index];

            if current_variant_length < max_valid_variant_length {
                // We need to add alignment between : and ty
                // max_valid_variant_length: the length of the variant that we are taking as a reference to align
                // current_variant_length: the length of the current variant that we are trying to format
                let required_alignment = max_valid_variant_length - current_variant_length;
                formatted_code.push_str(&(0..required_alignment).map(|_| ' ').collect::<String>());
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

            // TODO we are currently pushing \n directly, if we want to format storage
            // into a single line in some cases. We should handle this better!
            formatted_code.push(PunctKind::Comma.as_char());
            formatted_code.push('\n');
        }

        // Add `}`
        Self::close_curly_brace(formatted_code, formatter)?;

        Ok(())
    }
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
                line.push_str(&format!("\n{}\n", open_brace));
                shape = shape.block_indent(extra_width);
            }
            _ => {
                // Add opening brace to the same line
                line.push_str(&format!(" {}\n", open_brace));
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
