use crate::{
    config::{items::ItemBraceStyle, user_def::FieldAlignment},
    formatter::{shape::LineStyle, *},
    utils::{
        map::byte_span::{ByteSpan, LeafSpans},
        CurlyBrace,
    },
};
use std::fmt::Write;
use sway_ast::{token::Delimiter, ItemEnum};
use sway_types::Spanned;

#[cfg(test)]
mod tests;

impl Format for ItemEnum {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        formatter
            .shape
            .code_line
            .update_line_style(LineStyle::Multiline);
        // If there is a visibility token add it to the formatted_code with a ` ` after it.
        if let Some(visibility) = &self.visibility {
            write!(formatted_code, "{} ", visibility.span().as_str())?;
        }
        // Add enum token and name
        write!(formatted_code, "{} ", self.enum_token.span().as_str())?;
        self.name.format(formatted_code, formatter)?;
        // Format `GenericParams`, if any
        if let Some(generics) = &self.generics {
            generics.format(formatted_code, formatter)?;
        }

        let fields = self.fields.get();

        // Handle openning brace
        ItemEnum::open_curly_brace(formatted_code, formatter)?;
        // Determine alignment tactic
        match formatter.config.structures.field_alignment {
            FieldAlignment::AlignFields(enum_variant_align_threshold) => {
                writeln!(formatted_code)?;
                let value_pairs = &fields
                    .value_separator_pairs
                    .iter()
                    // TODO: Handle annotations instead of stripping them
                    .map(|pair| (&pair.0.value, &pair.1))
                    .collect::<Vec<_>>();
                // In first iteration we are going to be collecting the lengths of the enum variants.
                let variant_length: Vec<usize> = value_pairs
                    .iter()
                    .map(|(type_field, _)| type_field.name.as_str().len())
                    .collect();

                // Find the maximum length in the variant_length vector that is still smaller than enum_field_align_threshold.
                let mut max_valid_variant_length = 0;
                variant_length.iter().for_each(|length| {
                    if *length > max_valid_variant_length && *length < enum_variant_align_threshold
                    {
                        max_valid_variant_length = *length;
                    }
                });

                let mut value_pairs_iter = value_pairs.iter().enumerate().peekable();
                for (var_index, (type_field, comma_token)) in value_pairs_iter.clone() {
                    write!(
                        formatted_code,
                        "{}",
                        &formatter.shape.indent.to_string(&formatter.config)?
                    )?;

                    // Add name
                    type_field.name.format(formatted_code, formatter)?;
                    let current_variant_length = variant_length[var_index];
                    if current_variant_length < max_valid_variant_length {
                        // We need to add alignment between : and ty
                        // max_valid_variant_length: the length of the variant that we are taking as a reference to align
                        // current_variant_length: the length of the current variant that we are trying to format
                        let mut required_alignment =
                            max_valid_variant_length - current_variant_length;
                        while required_alignment != 0 {
                            write!(formatted_code, " ")?;
                            required_alignment -= 1;
                        }
                    }
                    // Add `:`, ty & `CommaToken`
                    write!(
                        formatted_code,
                        " {} ",
                        type_field.colon_token.span().as_str(),
                    )?;
                    type_field.ty.format(formatted_code, formatter)?;
                    if value_pairs_iter.peek().is_some() {
                        writeln!(formatted_code, "{}", comma_token.span().as_str())?;
                    } else if let Some(final_value) = &fields.final_value_opt {
                        // TODO: Handle annotation
                        let final_value = &final_value.value;
                        write!(formatted_code, "{}", final_value.span().as_str())?;
                    }
                }
            }
            FieldAlignment::Off => fields.format(formatted_code, formatter)?,
        }
        // Handle closing brace
        ItemEnum::close_curly_brace(formatted_code, formatter)?;
        formatter.shape.reset_line_settings();

        Ok(())
    }
}

impl CurlyBrace for ItemEnum {
    fn open_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let brace_style = formatter.config.items.item_brace_style;
        let open_brace = Delimiter::Brace.as_open_char();
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add openning brace to the next line.
                writeln!(line, "\n{}", open_brace)?;
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
        // If shape is becoming left-most aligned or - indent just have the defualt shape
        formatter.shape.block_unindent(&formatter.config);
        write!(
            line,
            "{}{}",
            formatter.shape.indent.to_string(&formatter.config)?,
            Delimiter::Brace.as_close_char()
        )?;

        Ok(())
    }
}
impl LeafSpans for ItemEnum {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        if let Some(visibility) = &self.visibility {
            collected_spans.push(ByteSpan::from(visibility.span()));
        }
        collected_spans.push(ByteSpan::from(self.enum_token.span()));
        collected_spans.push(ByteSpan::from(self.name.span()));
        if let Some(generics) = &self.generics {
            collected_spans.push(ByteSpan::from(generics.parameters.span()))
        }
        collected_spans.append(&mut self.fields.leaf_spans());
        collected_spans
    }
}
