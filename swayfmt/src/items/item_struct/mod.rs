use crate::{
    comments::{rewrite_with_comments, write_comments},
    config::user_def::FieldAlignment,
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
use sway_ast::{
    keywords::{ColonToken, Keyword, StructToken, Token},
    CommaToken, ItemStruct, PubToken,
};
use sway_types::{ast::Delimiter, Spanned};

#[cfg(test)]
mod tests;

impl Format for ItemStruct {
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
                // Required for comment formatting
                let start_len = formatted_code.len();
                // If there is a visibility token add it to the formatted_code with a ` ` after it.
                if self.visibility.is_some() {
                    write!(formatted_code, "{} ", PubToken::AS_STR)?;
                }
                // Add struct token and name
                write!(formatted_code, "{} ", StructToken::AS_STR)?;
                self.name.format(formatted_code, formatter)?;
                // Format `GenericParams`, if any
                if let Some(generics) = &self.generic_params_opt {
                    generics.format(formatted_code, formatter)?;
                }
                if let Some(where_clause) = &self.where_clause_opt {
                    writeln!(formatted_code)?;
                    where_clause.format(formatted_code, formatter)?;
                    formatter.shape.code_line.update_where_clause(true);
                }

                let fields = self.fields.get();

                // Handle opening brace
                Self::open_curly_brace(formatted_code, formatter)?;

                if fields.is_empty() {
                    write_comments(formatted_code, self.span().into(), formatter)?;
                }

                formatter.shape.code_line.update_expr_new_line(true);

                // Determine alignment tactic
                match formatter.config.structures.field_alignment {
                    FieldAlignment::AlignFields(struct_field_align_threshold) => {
                        writeln!(formatted_code)?;
                        let type_fields = &fields
                            .value_separator_pairs
                            .iter()
                            // TODO: Handle annotations instead of stripping them.
                            //       See: https://github.com/FuelLabs/sway/issues/6802
                            .map(|(type_field, _comma_token)| &type_field.value)
                            .collect::<Vec<_>>();
                        // In first iteration we are going to be collecting the lengths of the struct fields.
                        // We need to include the `pub` keyword in the length, if the field is public,
                        // together with one space character between the `pub` and the name.
                        let fields_lengths: Vec<usize> = type_fields
                            .iter()
                            .map(|type_field| {
                                type_field
                                    .visibility
                                    .as_ref()
                                    .map_or(0, |_pub_token| PubToken::AS_STR.len() + 1)
                                    + type_field.name.as_str().len()
                            })
                            .collect();

                        // Find the maximum length that is still smaller than the align threshold.
                        let mut max_valid_field_length = 0;
                        fields_lengths.iter().for_each(|length| {
                            if *length > max_valid_field_length
                                && *length < struct_field_align_threshold
                            {
                                max_valid_field_length = *length;
                            }
                        });

                        for (var_index, type_field) in type_fields.iter().enumerate() {
                            write!(formatted_code, "{}", formatter.indent_to_str()?)?;
                            // If there is a visibility token add it to the formatted_code with a ` ` after it.
                            if type_field.visibility.is_some() {
                                write!(formatted_code, "{} ", PubToken::AS_STR)?;
                            }
                            // Add name
                            type_field.name.format(formatted_code, formatter)?;
                            let current_field_length = fields_lengths[var_index];
                            if current_field_length < max_valid_field_length {
                                // We need to add alignment between : and ty
                                // max_valid_variant_length: the length of the variant that we are taking as a reference to align
                                // current_variant_length: the length of the current variant that we are trying to format
                                let mut required_alignment =
                                    max_valid_field_length - current_field_length;
                                while required_alignment != 0 {
                                    write!(formatted_code, " ")?;
                                    required_alignment -= 1;
                                }
                            }
                            // Add `:`, ty & `CommaToken`
                            write!(formatted_code, " {} ", ColonToken::AS_STR)?;
                            type_field.ty.format(formatted_code, formatter)?;
                            writeln!(formatted_code, "{}", CommaToken::AS_STR)?;
                        }
                        if let Some(final_value) = &fields.final_value_opt {
                            final_value.format(formatted_code, formatter)?;
                            writeln!(formatted_code)?;
                        }
                    }
                    FieldAlignment::Off => {
                        fields.format(formatted_code, formatter)?;
                    }
                }
                // Handle closing brace
                Self::close_curly_brace(formatted_code, formatter)?;

                rewrite_with_comments::<ItemStruct>(
                    formatter,
                    self.span(),
                    self.leaf_spans(),
                    formatted_code,
                    start_len,
                )?;
                Ok(())
            },
        )?;

        Ok(())
    }
}

impl CurlyBrace for ItemStruct {
    fn open_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        formatter.indent();
        let open_brace = Delimiter::Brace.as_open_char();
        match formatter.shape.code_line.has_where_clause {
            true => {
                write!(line, "{open_brace}")?;
                formatter.shape.code_line.update_where_clause(false);
            }
            false => {
                write!(line, " {open_brace}")?;
            }
        }

        Ok(())
    }

    fn close_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // If shape is becoming left-most aligned or - indent just have the default shape
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

impl LeafSpans for ItemStruct {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        if let Some(visibility) = &self.visibility {
            collected_spans.push(ByteSpan::from(visibility.span()));
        }
        collected_spans.push(ByteSpan::from(self.struct_token.span()));
        collected_spans.push(ByteSpan::from(self.name.span()));
        if let Some(generics) = &self.generic_params_opt {
            collected_spans.push(ByteSpan::from(generics.parameters.span()))
        }
        collected_spans.append(&mut self.fields.leaf_spans());
        collected_spans
    }
}
