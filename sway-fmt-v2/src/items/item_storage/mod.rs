use crate::{
    config::{items::ItemBraceStyle, user_def::FieldAlignment},
    formatter::{shape::LineStyle, *},
    utils::{
        map::byte_span::{ByteSpan, LeafSpans},
        CurlyBrace,
    },
};
use std::fmt::Write;
use sway_ast::{keywords::Token, token::Delimiter, ItemStorage, StorageField};
use sway_types::Spanned;

#[cfg(test)]
mod tests;

impl Format for ItemStorage {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        formatter
            .shape
            .code_line
            .update_line_style(LineStyle::Multiline);
        // Add storage token
        write!(formatted_code, "{}", self.storage_token.span().as_str())?;
        let fields = self.fields.get();

        // Handle openning brace
        Self::open_curly_brace(formatted_code, formatter)?;

        // Determine alignment tactic
        match formatter.config.structures.field_alignment {
            FieldAlignment::AlignFields(storage_field_align_threshold) => {
                writeln!(formatted_code)?;
                let value_pairs = &fields
                    .value_separator_pairs
                    .iter()
                    // TODO: Handle annotations instead of stripping them
                    .map(|pair| (&pair.0.value, &pair.1))
                    .collect::<Vec<_>>();
                // In first iteration we are going to be collecting the lengths of the struct fields.
                let field_length: Vec<usize> = value_pairs
                    .iter()
                    .map(|(storage_field, _)| storage_field.name.as_str().len())
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
                for (field_index, (storage_field, comma_token)) in value_pairs_iter.clone() {
                    write!(
                        formatted_code,
                        "{}",
                        &formatter.shape.indent.to_string(&formatter.config)?
                    )?;

                    // Add name
                    write!(formatted_code, "{}", storage_field.name.as_str())?;

                    // `current_field_length`: the length of the current field that we are trying to format.
                    let current_field_length = field_length[field_index];
                    if current_field_length < max_valid_field_length {
                        // We need to add alignment between `:` and `ty`
                        let mut required_alignment = max_valid_field_length - current_field_length;
                        while required_alignment != 0 {
                            write!(formatted_code, " ")?;
                            required_alignment -= 1;
                        }
                    }
                    // Add `:`, `ty` & `CommaToken`
                    write!(
                        formatted_code,
                        " {} ",
                        storage_field.colon_token.ident().as_str(),
                    )?;
                    storage_field.ty.format(formatted_code, formatter)?;
                    write!(
                        formatted_code,
                        " {} ",
                        storage_field.eq_token.ident().as_str()
                    )?;
                    storage_field
                        .initializer
                        .format(formatted_code, formatter)?;
                    if value_pairs_iter.peek().is_some() {
                        writeln!(formatted_code, "{}", comma_token.ident().as_str())?;
                    } else if let Some(final_value) = &fields.final_value_opt {
                        final_value.format(formatted_code, formatter)?;
                    }
                }
            }
            FieldAlignment::Off => fields.format(formatted_code, formatter)?,
        }
        // Handle closing brace
        Self::close_curly_brace(formatted_code, formatter)?;
        formatter.shape.reset_line_settings();

        Ok(())
    }
}

impl CurlyBrace for ItemStorage {
    fn open_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let brace_style = formatter.config.items.item_brace_style;
        let open_brace = Delimiter::Brace.as_open_char();
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add opening brace to the next line.
                write!(line, "\n{}", open_brace)?;
                formatter.shape.block_indent(&formatter.config);
            }
            _ => {
                // Add opening brace to the same line
                write!(line, " {}", open_brace)?;
                formatter.shape.block_indent(&formatter.config);
            }
        }

        Ok(())
    }
    fn close_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Brace.as_close_char())?;
        // shrink_left would return error if the current indentation level is becoming < 0, in that
        // case we should use the Shape::default() which has 0 indentation level.
        formatter.shape.block_unindent(&formatter.config);

        Ok(())
    }
}

impl LeafSpans for ItemStorage {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.storage_token.span())];
        collected_spans.append(&mut self.fields.leaf_spans());
        collected_spans
    }
}

impl LeafSpans for StorageField {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.name.span())];
        collected_spans.push(ByteSpan::from(self.colon_token.span()));
        collected_spans.append(&mut self.ty.leaf_spans());
        collected_spans.push(ByteSpan::from(self.eq_token.span()));
        collected_spans.append(&mut self.initializer.leaf_spans());
        collected_spans
    }
}
