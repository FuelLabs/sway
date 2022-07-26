use crate::{
    config::{items::ItemBraceStyle, user_def::FieldAlignment},
    fmt::{Format, FormattedCode, Formatter},
    utils::{
        bracket::CurlyBrace,
        comments::{ByteSpan, LeafSpans},
        item::ItemLenChars,
    },
    FormatterError,
};
use std::fmt::Write;
use sway_parse::{
    token::{Delimiter, PunctKind},
    ItemStruct,
};
use sway_types::Spanned;

impl Format for ItemStruct {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // Bring configurations into scope.
        //
        // Should small structs formatted into a single line.
        let struct_lit_single_line = formatter.config.structures.small_structures_single_line;

        // Get the width limit of a struct to be formatted into single line if struct_lit_single_line is true.
        let width_heuristics = formatter
            .config
            .heuristics
            .heuristics_pref
            .to_width_heuristics(&formatter.config.whitespace);
        let struct_lit_width = width_heuristics.structure_lit_width;

        let multiline = !struct_lit_single_line || self.len_chars()? > struct_lit_width;

        format_struct(self, formatted_code, formatter, multiline)?;
        Ok(())
    }
}

/// Format the struct if the multiline is passed as false struct will be formatted into a single line.
///
/// ## Examples
///
/// (multiline : false):
///
/// ```rust,ignore
/// struct Foo { bar: u64,  baz: bool }
/// ```
///
/// (multiline : true):
///
/// ```rust,ignore
/// struct Foo {
///     bar: u64,
///     baz: bool,
/// }
/// ```
fn format_struct(
    item_struct: &ItemStruct,
    formatted_code: &mut FormattedCode,
    formatter: &mut Formatter,
    multiline: bool,
) -> Result<(), FormatterError> {
    // If there is a visibility token add it to the formatted_code with a ` ` after it.
    if let Some(visibility) = &item_struct.visibility {
        write!(formatted_code, "{} ", visibility.span().as_str())?;
    }
    // Add struct token and name
    write!(
        formatted_code,
        "{} {}",
        item_struct.struct_token.span().as_str(),
        item_struct.name.as_str(),
    )?;

    // Format `GenericParams`, if any
    if let Some(generics) = &item_struct.generics {
        generics.format(formatted_code, formatter)?;
    }

    let fields = item_struct.fields.clone().into_inner();

    // Handle openning brace
    ItemStruct::open_curly_brace(formatted_code, formatter)?;
    if multiline {
        writeln!(formatted_code)?;
        // Determine alignment tactic
        match formatter.config.structures.field_alignment {
            FieldAlignment::AlignFields(struct_field_align_threshold) => {
                let value_pairs = fields.value_separator_pairs;
                // In first iteration we are going to be collecting the lengths of the struct fields.
                let field_length: Vec<usize> = value_pairs
                    .iter()
                    .map(|field| field.0.name.as_str().len())
                    .collect();

                // Find the maximum length in the `field_length` vector that is still smaller than `struct_field_align_threshold`.
                // `max_valid_field_length`: the length of the field that we are taking as a reference to align.
                let mut max_valid_field_length = 0;
                field_length.iter().for_each(|length| {
                    if *length > max_valid_field_length && *length < struct_field_align_threshold {
                        max_valid_field_length = *length;
                    }
                });

                let mut value_pairs_iter = value_pairs.iter().enumerate().peekable();
                for (field_index, field) in value_pairs_iter.clone() {
                    write!(formatted_code, "{}", &formatter.shape.to_string(formatter)?)?;

                    let type_field = &field.0;
                    // Add name
                    write!(formatted_code, "{}", type_field.name.as_str())?;

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
                        type_field.colon_token.ident().as_str(),
                    )?;
                    type_field.ty.format(formatted_code, formatter)?;
                    if value_pairs_iter.peek().is_some() {
                        writeln!(formatted_code, "{}", field.1.span().as_str())?;
                    } else if let Some(final_value) = &fields.final_value_opt {
                        write!(formatted_code, "{}", final_value.span().as_str())?;
                    }
                }
            }
            FieldAlignment::Off => {
                let mut value_pairs_iter = fields.value_separator_pairs.iter().peekable();
                for field in value_pairs_iter.clone() {
                    write!(formatted_code, "{}", &formatter.shape.to_string(formatter)?)?;
                    // TypeField
                    field.0.format(formatted_code, formatter)?;

                    if value_pairs_iter.peek().is_some() {
                        writeln!(formatted_code, "{}", field.1.span().as_str())?;
                    }
                }
                if let Some(final_value) = &fields.final_value_opt {
                    write!(formatted_code, "{}", &formatter.shape.to_string(formatter)?)?;
                    final_value.format(formatted_code, formatter)?;
                    writeln!(formatted_code, "{}", PunctKind::Comma.as_char())?;
                }
            }
        }
    } else {
        // non-multiline formatting
        write!(formatted_code, " ")?;
        let mut value_pairs_iter = fields.value_separator_pairs.iter().peekable();
        for field in value_pairs_iter.clone() {
            // TypeField
            field.0.format(formatted_code, formatter)?;

            if value_pairs_iter.peek().is_some() {
                write!(formatted_code, "{} ", field.1.span().as_str())?;
            }
        }
        if let Some(final_value) = &fields.final_value_opt {
            final_value.format(formatted_code, formatter)?;
            write!(formatted_code, " ")?;
        } else {
            formatted_code.pop();
            formatted_code.pop();
            write!(formatted_code, " ")?;
        }
    }

    // Handle closing brace
    ItemStruct::close_curly_brace(formatted_code, formatter)?;
    Ok(())
}

impl ItemLenChars for ItemStruct {
    fn len_chars(&self) -> Result<usize, FormatterError> {
        // Format to single line and return the length
        let mut str_item = String::new();
        let mut formatter = Formatter::default();
        format_struct(self, &mut str_item, &mut formatter, false)?;
        Ok(str_item.chars().count() as usize)
    }
}

impl CurlyBrace for ItemStruct {
    fn open_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let brace_style = formatter.config.items.item_brace_style;
        let mut shape = formatter.shape;
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add openning brace to the next line.
                write!(line, "\n{}", Delimiter::Brace.as_open_char())?;
                shape = shape.block_indent(formatter);
            }
            _ => {
                // Add opening brace to the same line
                write!(line, " {}", Delimiter::Brace.as_open_char())?;
                shape = shape.block_indent(formatter);
            }
        }

        formatter.shape = shape;
        Ok(())
    }

    fn close_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Brace.as_close_char())?;
        // If shape is becoming left-most alligned or - indent just have the defualt shape
        formatter.shape = formatter.shape.block_unindent(formatter);
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
        if let Some(generics) = &self.generics {
            collected_spans.push(ByteSpan::from(generics.parameters.span()))
        }
        collected_spans.append(&mut self.fields.leaf_spans());
        collected_spans
    }
}
