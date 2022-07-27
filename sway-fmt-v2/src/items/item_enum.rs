use crate::{
    config::{items::ItemBraceStyle, user_def::FieldAlignment},
    fmt::{Format, FormattedCode, Formatter},
    utils::{
        bracket::CurlyBrace,
        comments::{ByteSpan, LeafSpans},
        indent_style::LineStyle,
        item::ItemLenChars,
    },
    FormatterError,
};
use std::fmt::Write;
use sway_parse::{
    token::{Delimiter, PunctKind},
    ItemEnum,
};
use sway_types::Spanned;

impl Format for ItemEnum {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let mut shape = formatter.shape;
        shape = shape.update_width(self.len_chars()?);
        formatter.shape = shape.get_line_style(formatter);

        format_enum(self, formatted_code, formatter)?;
        if formatter.shape.line_style == LineStyle::Inline {
            formatter.shape = shape.reset_line_style();
        }

        Ok(())
    }
}

/// Format the enum if the multiline is passed as false enum will be formatted into a single line.
///
/// ##examples
///
/// (multiline : false):
///
/// ```rust,ignore
/// enum Foo { bar: u64,  baz: bool }
/// ```
///
/// (multiline : true):
/// ```rust,ignore
/// enum Foo {
///     bar: u64,
///     baz: bool,
/// }
/// ```
fn format_enum(
    item_enum: &ItemEnum,
    formatted_code: &mut FormattedCode,
    formatter: &mut Formatter,
) -> Result<(), FormatterError> {
    // If there is a visibility token add it to the formatted_code with a ` ` after it.
    if let Some(visibility) = &item_enum.visibility {
        write!(formatted_code, "{} ", visibility.span().as_str())?;
    }
    // Add enum token and name
    write!(
        formatted_code,
        "{} {}",
        item_enum.enum_token.span().as_str(),
        item_enum.name.as_str()
    )?;

    // Format `GenericParams`, if any
    if let Some(generics) = &item_enum.generics {
        generics.format(formatted_code, formatter)?;
    }

    let fields = item_enum.fields.clone().into_inner();

    // Handle openning brace
    ItemEnum::open_curly_brace(formatted_code, formatter)?;
    match formatter.shape.line_style {
        LineStyle::Multiline => {
            writeln!(formatted_code)?;
            // Determine alignment tactic
            match formatter.config.structures.field_alignment {
                FieldAlignment::AlignFields(enum_variant_align_threshold) => {
                    let value_pairs = fields.value_separator_pairs;
                    // In first iteration we are going to be collecting the lengths of the enum variants.
                    let variant_length: Vec<usize> = value_pairs
                        .iter()
                        .map(|variant| variant.0.name.as_str().len())
                        .collect();

                    // Find the maximum length in the variant_length vector that is still smaller than enum_field_align_threshold.
                    let mut max_valid_variant_length = 0;
                    variant_length.iter().for_each(|length| {
                        if *length > max_valid_variant_length
                            && *length < enum_variant_align_threshold
                        {
                            max_valid_variant_length = *length;
                        }
                    });

                    let mut value_pairs_iter = value_pairs.iter().enumerate().peekable();
                    for (var_index, variant) in value_pairs_iter.clone() {
                        write!(formatted_code, "{}", &formatter.shape.to_string(formatter)?)?;

                        let type_field = &variant.0;
                        // Add name
                        write!(formatted_code, "{}", type_field.name.as_str())?;
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
                            type_field.colon_token.ident().as_str(),
                        )?;
                        type_field.ty.format(formatted_code, formatter)?;
                        if value_pairs_iter.peek().is_some() {
                            writeln!(formatted_code, "{}", variant.1.span().as_str())?;
                        } else if let Some(final_value) = &fields.final_value_opt {
                            write!(formatted_code, "{}", final_value.span().as_str())?;
                        }
                    }
                }
                FieldAlignment::Off => {
                    let mut value_pairs_iter = fields.value_separator_pairs.iter().peekable();
                    for variant in value_pairs_iter.clone() {
                        write!(formatted_code, "{}", &formatter.shape.to_string(formatter)?)?;
                        // TypeField
                        variant.0.format(formatted_code, formatter)?;

                        if value_pairs_iter.peek().is_some() {
                            writeln!(formatted_code, "{}", variant.1.span().as_str())?;
                        }
                    }
                    if let Some(final_value) = &fields.final_value_opt {
                        write!(formatted_code, "{}", &formatter.shape.to_string(formatter)?)?;
                        final_value.format(formatted_code, formatter)?;
                        writeln!(formatted_code, "{}", PunctKind::Comma.as_char())?;
                    }
                }
            }
        }
        LineStyle::Inline => {
            // non-multiline formatting
            write!(formatted_code, " ")?;
            let mut value_pairs_iter = fields.value_separator_pairs.iter().peekable();
            for variant in value_pairs_iter.clone() {
                variant.0.format(formatted_code, formatter)?;

                if value_pairs_iter.peek().is_some() {
                    write!(formatted_code, "{} ", variant.1.span().as_str())?;
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
    }
    // Handle closing brace
    ItemEnum::close_curly_brace(formatted_code, formatter)?;

    Ok(())
}

impl ItemLenChars for ItemEnum {
    fn len_chars(&self) -> Result<usize, FormatterError> {
        // Format to single line and return the length
        let mut str_item = String::new();
        let mut formatter = Formatter::default();
        formatter.shape.line_style = LineStyle::Inline;
        format_enum(self, &mut str_item, &mut formatter)?;
        Ok(str_item.chars().count() as usize)
    }
}

impl CurlyBrace for ItemEnum {
    fn open_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let brace_style = formatter.config.items.item_brace_style;
        let mut shape = formatter.shape;
        let open_brace = Delimiter::Brace.as_open_char();
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add openning brace to the next line.
                writeln!(line, "\n{}", open_brace)?;
                shape = shape.block_indent(formatter);
            }
            _ => {
                // Add opening brace to the same line
                write!(line, " {}", open_brace)?;
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
        // If shape is becoming left-most aligned or - indent just have the defualt shape
        formatter.shape = formatter.shape.block_unindent(formatter);
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
