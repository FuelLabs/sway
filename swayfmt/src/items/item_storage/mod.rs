use crate::{
    comments::rewrite_with_comments,
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
use std::{collections::HashMap, fmt::Write};
use sway_ast::{keywords::Token, ItemStorage, StorageEntry, StorageField};
use sway_types::{ast::Delimiter, IdentUnique, Spanned};

#[cfg(test)]
mod tests;

impl Format for ItemStorage {
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

                // Add storage token
                write!(formatted_code, "{}", self.storage_token.span().as_str())?;
                let entries = self.entries.get();

                // Handle opening brace
                Self::open_curly_brace(formatted_code, formatter)?;

                formatter.shape.code_line.update_expr_new_line(true);

                // Determine alignment tactic
                match formatter.config.structures.field_alignment {
                    FieldAlignment::AlignFields(storage_field_align_threshold) => {
                        writeln!(formatted_code)?;
                        let value_pairs = &entries
                            .value_separator_pairs
                            .iter()
                            // TODO: Handle annotations instead of stripping them
                            .map(|(storage_field, comma_token)| (&storage_field.value, comma_token))
                            .collect::<Vec<_>>();
                        // In first iteration we are going to be collecting the lengths of the
                        // struct fields.
                        let mut field_lengths: HashMap<IdentUnique, usize> =
                            HashMap::<IdentUnique, usize>::new();
                        fn collect_field_lengths(
                            entry: &StorageEntry,
                            ident_size: usize,
                            current_ident: usize,
                            field_lengths: &mut HashMap<IdentUnique, usize>,
                        ) {
                            if let Some(namespace) = &entry.namespace {
                                namespace.clone().into_inner().into_iter().for_each(|e| {
                                    collect_field_lengths(
                                        &e.value,
                                        ident_size,
                                        current_ident + ident_size,
                                        field_lengths,
                                    )
                                });
                            } else if let Some(storage_field) = &entry.field {
                                field_lengths.insert(
                                    storage_field.name.clone().into(),
                                    current_ident + storage_field.name.as_str().len(),
                                );
                            }
                        }
                        let ident_size = formatter.config.whitespace.tab_spaces;
                        value_pairs.iter().for_each(|(storage_entry, _)| {
                            collect_field_lengths(storage_entry, ident_size, 0, &mut field_lengths)
                        });
                        if let Some(final_value) = &entries.final_value_opt {
                            collect_field_lengths(
                                &final_value.value,
                                ident_size,
                                0,
                                &mut field_lengths,
                            );
                        }

                        // Find the maximum length in the `field_length` vector that is still
                        // smaller than `storage_field_align_threshold`.  `max_valid_field_length`:
                        // the length of the field that we are taking as a reference to align.
                        let mut max_valid_field_length = 0;
                        field_lengths.iter().for_each(|(_, length)| {
                            if *length > max_valid_field_length
                                && *length < storage_field_align_threshold
                            {
                                max_valid_field_length = *length;
                            }
                        });

                        fn format_entry(
                            formatted_code: &mut FormattedCode,
                            formatter: &mut Formatter,
                            entry: &StorageEntry,
                            field_lengths: &HashMap<IdentUnique, usize>,
                            max_valid_field_length: usize,
                        ) -> Result<(), FormatterError> {
                            write!(formatted_code, "{}", formatter.indent_to_str()?)?;
                            if let Some(namespace) = &entry.namespace {
                                entry.name.format(formatted_code, formatter)?;
                                ItemStorage::open_curly_brace(formatted_code, formatter)?;
                                writeln!(formatted_code)?;

                                for (e, comma_token) in
                                    namespace.clone().into_inner().value_separator_pairs
                                {
                                    format_entry(
                                        formatted_code,
                                        formatter,
                                        &e.value,
                                        field_lengths,
                                        max_valid_field_length,
                                    )?;
                                    writeln!(formatted_code, "{}", comma_token.ident().as_str())?;
                                }
                                if let Some(final_value) =
                                    &namespace.clone().into_inner().final_value_opt
                                {
                                    format_entry(
                                        formatted_code,
                                        formatter,
                                        &final_value.value,
                                        field_lengths,
                                        max_valid_field_length,
                                    )?;
                                    writeln!(formatted_code)?;
                                }

                                ItemStorage::close_curly_brace(formatted_code, formatter)?;
                            } else if let Some(storage_field) = &entry.field {
                                // Add name
                                storage_field.name.format(formatted_code, formatter)?;

                                // `current_field_length`: the length of the current field that we are
                                // trying to format.
                                let current_field_length = field_lengths
                                    .get(&storage_field.name.clone().into())
                                    .unwrap();
                                if *current_field_length < max_valid_field_length {
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
                            }

                            Ok(())
                        }
                        for (storage_entry, comma_token) in value_pairs.iter().clone() {
                            format_entry(
                                formatted_code,
                                formatter,
                                storage_entry,
                                &field_lengths,
                                max_valid_field_length,
                            )?;
                            writeln!(formatted_code, "{}", comma_token.ident().as_str())?;
                        }
                        if let Some(final_value) = &entries.final_value_opt {
                            format_entry(
                                formatted_code,
                                formatter,
                                &final_value.value,
                                &field_lengths,
                                max_valid_field_length,
                            )?;
                            writeln!(formatted_code)?;
                        }
                    }
                    FieldAlignment::Off => entries.format(formatted_code, formatter)?,
                }

                // Handle closing brace
                Self::close_curly_brace(formatted_code, formatter)?;

                rewrite_with_comments::<ItemStorage>(
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

impl CurlyBrace for ItemStorage {
    fn open_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        formatter.indent();
        let open_brace = Delimiter::Brace.as_open_char();
        // Add opening brace to the same line
        write!(line, " {open_brace}")?;

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

impl LeafSpans for ItemStorage {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.storage_token.span())];
        collected_spans.append(&mut self.entries.leaf_spans());
        collected_spans
    }
}

impl LeafSpans for StorageEntry {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        if let Some(namespace) = &self.namespace {
            let mut collected_spans = vec![ByteSpan::from(self.name.span())];
            collected_spans.append(&mut namespace.leaf_spans());
            collected_spans
        } else if let Some(field) = &self.field {
            field.leaf_spans()
        } else {
            vec![]
        }
    }
}

impl LeafSpans for StorageField {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.name.span())];
        if let Some(in_token) = &self.in_token {
            collected_spans.push(ByteSpan::from(in_token.span()));
        }
        if let Some(key_expr) = &self.key_expr {
            collected_spans.push(ByteSpan::from(key_expr.span()));
        }
        collected_spans.push(ByteSpan::from(self.colon_token.span()));
        collected_spans.append(&mut self.ty.leaf_spans());
        collected_spans.push(ByteSpan::from(self.eq_token.span()));
        collected_spans.append(&mut self.initializer.leaf_spans());
        collected_spans
    }
}
