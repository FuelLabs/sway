use crate::{
    config::{items::ItemBraceStyle, user_def::FieldAlignment},
    formatter::{
        shape::{ExprKind, LineStyle},
        *,
    },
    utils::{
        map::byte_span::{ByteSpan, LeafSpans},
        CurlyBrace,
    },
};
use std::fmt::Write;
use sway_ast::{keywords::Token, ConfigurableField, ItemConfigurable};
use sway_types::{ast::Delimiter, Spanned};

#[cfg(test)]
mod tests;

impl Format for ItemConfigurable {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        formatter.with_shape(
            formatter
                .shape
                .with_code_line_from(LineStyle::Multiline, ExprKind::default()),
            |formatter| -> Result<(), FormatterError> {
                // Add configurable token
                write!(
                    formatted_code,
                    "{}",
                    self.configurable_token.span().as_str()
                )?;
                let fields = self.fields.get();

                // Handle opening brace
                Self::open_curly_brace(formatted_code, formatter)?;

                // Determine alignment tactic
                match formatter.config.structures.field_alignment {
                    FieldAlignment::AlignFields(configurable_field_align_threshold) => {
                        writeln!(formatted_code)?;
                        let value_pairs = &fields
                            .value_separator_pairs
                            .iter()
                            // TODO: Handle annotations instead of stripping them
                            .map(|(configurable_field, comma_token)| {
                                (&configurable_field.value, comma_token)
                            })
                            .collect::<Vec<_>>();
                        // In first iteration we are going to be collecting the lengths of the
                        // struct fields.
                        let field_length: Vec<usize> = value_pairs
                            .iter()
                            .map(|(configurable_field, _)| configurable_field.name.as_str().len())
                            .collect();

                        // Find the maximum length in the `field_length` vector that is still
                        // smaller than `configurable_field_align_threshold`.
                        // `max_valid_field_length`: the length of the field that we are taking as
                        // a reference to align.
                        let mut max_valid_field_length = 0;
                        field_length.iter().for_each(|length| {
                            if *length > max_valid_field_length
                                && *length < configurable_field_align_threshold
                            {
                                max_valid_field_length = *length;
                            }
                        });

                        let value_pairs_iter = value_pairs.iter().enumerate();
                        for (field_index, (configurable_field, comma_token)) in
                            value_pairs_iter.clone()
                        {
                            write!(formatted_code, "{}", &formatter.indent_to_str()?)?;

                            // Add name
                            configurable_field.name.format(formatted_code, formatter)?;

                            // `current_field_length`: the length of the current field that we are
                            // trying to format.
                            let current_field_length = field_length[field_index];
                            if current_field_length < max_valid_field_length {
                                // We need to add alignment between `:` and `ty`
                                let mut required_alignment =
                                    max_valid_field_length - current_field_length;
                                while required_alignment != 0 {
                                    write!(formatted_code, " ")?;
                                    required_alignment -= 1;
                                }
                            }
                            // Add `:`, `ty` & `CommaToken`
                            write!(
                                formatted_code,
                                " {} ",
                                configurable_field.colon_token.ident().as_str(),
                            )?;
                            configurable_field.ty.format(formatted_code, formatter)?;
                            write!(
                                formatted_code,
                                " {} ",
                                configurable_field.eq_token.ident().as_str()
                            )?;
                            configurable_field
                                .initializer
                                .format(formatted_code, formatter)?;
                            writeln!(formatted_code, "{}", comma_token.ident().as_str())?;
                        }
                        if let Some(final_value) = &fields.final_value_opt {
                            final_value.format(formatted_code, formatter)?;
                        }
                    }
                    FieldAlignment::Off => fields.format(formatted_code, formatter)?,
                }
                // Handle closing brace
                Self::close_curly_brace(formatted_code, formatter)?;

                Ok(())
            },
        )?;

        Ok(())
    }
}

impl CurlyBrace for ItemConfigurable {
    fn open_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let brace_style = formatter.config.items.item_brace_style;
        formatter.indent();
        let open_brace = Delimiter::Brace.as_open_char();
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add opening brace to the next line.
                write!(line, "\n{open_brace}")?;
            }
            _ => {
                // Add opening brace to the same line
                write!(line, " {open_brace}")?;
            }
        }

        Ok(())
    }
    fn close_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // shrink_left would return error if the current indentation level is becoming < 0, in that
        // case we should use the Shape::default() which has 0 indentation level.
        formatter.unindent();
        write!(
            line,
            "{}{}",
            formatter.indent_to_str()?,
            Delimiter::Brace.as_close_char()
        )?;

        Ok(())
    }
}

impl LeafSpans for ItemConfigurable {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.configurable_token.span())];
        collected_spans.append(&mut self.fields.leaf_spans());
        collected_spans
    }
}

impl LeafSpans for ConfigurableField {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.name.span())];
        collected_spans.push(ByteSpan::from(self.colon_token.span()));
        collected_spans.append(&mut self.ty.leaf_spans());
        collected_spans.push(ByteSpan::from(self.eq_token.span()));
        collected_spans.append(&mut self.initializer.leaf_spans());
        collected_spans
    }
}
