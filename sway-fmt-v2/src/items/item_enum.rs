use crate::{
    config::{items::ItemBraceStyle, user_def::FieldAlignment},
    fmt::{Format, FormattedCode, Formatter},
    utils::{bracket::CurlyBrace, item_len::ItemLen},
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
        // Bring configurations into scope.
        //
        // Should small enums formatted into a single line.
        let enum_lit_single_line = formatter.config.structures.small_structures_single_line;

        // Get the width limit of an enum to be formatted into single line if `enum_lit_single_line` is true.
        let width_heuristics = formatter
            .config
            .heuristics
            .heuristics_pref
            .to_width_heuristics(&formatter.config.whitespace);
        let enum_lit_width = width_heuristics.structure_lit_width;

        let multiline = !enum_lit_single_line || self.get_formatted_len() > enum_lit_width;

        format_enum(self, formatted_code, formatter, multiline)?;
        Ok(())
    }
}

/// Format the enum if the multiline is passed as false enum will be formatted into a single line.
///
/// Example (multiline : false):
/// enum Foo { bar: u64,  baz: bool }
///
/// Example (multiline : true):
/// enum Foo {
///     bar: u64,
///     baz: bool,
/// }
fn format_enum(
    item_enum: &ItemEnum,
    formatted_code: &mut FormattedCode,
    formatter: &mut Formatter,
    multiline: bool,
) -> Result<(), FormatterError> {
    // If there is a visibility token add it to the formatted_code with a ` ` after it.
    if let Some(visibility) = &item_enum.visibility {
        write!(formatted_code, "{} ", visibility.span().as_str())?;
    }
    // Add enum token
    write!(formatted_code, "{} ", item_enum.enum_token.span().as_str())?;

    // Add enum name
    formatted_code.push_str(item_enum.name.as_str());

    // Format `GenericParams`, if any
    if let Some(generics) = &item_enum.generics {
        generics.format(formatted_code, formatter)?;
    }

    let variants = item_enum.fields.clone().into_inner();

    // Handle openning brace
    ItemEnum::open_curly_brace(formatted_code, formatter)?;
    if multiline {
        formatted_code.push('\n');
        // Determine alignment tactic
        match formatter.config.structures.field_alignment {
            FieldAlignment::AlignFields(enum_variant_align_threshold) => {
                let value_pairs = variants.value_separator_pairs;
                // In first iteration we are going to be collecting the lengths of the enum variants.
                let variant_length: Vec<usize> = value_pairs
                    .iter()
                    .map(|variant| variant.0.name.as_str().len())
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
                for (var_index, variant) in value_pairs_iter.clone() {
                    formatted_code.push_str(&formatter.shape.indent.to_string(formatter));

                    let type_field = &variant.0;
                    // Add name
                    formatted_code.push_str(type_field.name.as_str());
                    let current_variant_length = variant_length[var_index];
                    if current_variant_length < max_valid_variant_length {
                        // We need to add alignment between : and ty
                        // max_valid_variant_length: the length of the variant that we are taking as a reference to align
                        // current_variant_length: the length of the current variant that we are trying to format
                        let mut required_alignment =
                            max_valid_variant_length - current_variant_length;
                        while required_alignment != 0 {
                            formatted_code.push(' ');
                            required_alignment -= 1;
                        }
                    }
                    // Add `:`, ty & `CommaToken`
                    //
                    // TODO(#2101): We are currently converting ty to string directly but we will probably need to format ty before adding.
                    write!(
                        formatted_code,
                        " {} {}",
                        type_field.colon_token.ident().as_str(),
                        type_field.ty.span().as_str(),
                    )?;
                    if value_pairs_iter.peek().is_some() {
                        writeln!(formatted_code, "{}", variant.1.span().as_str())?;
                    } else if let Some(final_value) = &variants.final_value_opt {
                        formatted_code.push_str(final_value.span().as_str());
                    }
                }
            }
            FieldAlignment::Off => {
                let mut value_pairs_iter = variants.value_separator_pairs.iter().peekable();
                for variant in value_pairs_iter.clone() {
                    formatted_code.push_str(&formatter.shape.indent.to_string(formatter));
                    let item_field = &variant.0;
                    item_field.format(formatted_code, formatter)?;

                    if value_pairs_iter.peek().is_some() {
                        writeln!(formatted_code, "{}", variant.1.span().as_str())?;
                    }
                }
                if let Some(final_value) = &variants.final_value_opt {
                    formatted_code.push_str(&formatter.shape.indent.to_string(formatter));
                    final_value.format(formatted_code, formatter)?;
                    writeln!(formatted_code, "{}", PunctKind::Comma.as_char())?;
                }
            }
        }
    } else {
        // non-multiline formatting
        formatted_code.push(' ');
        let mut value_pairs_iter = variants.value_separator_pairs.iter().peekable();
        for variant in value_pairs_iter.clone() {
            let item_field = &variant.0;
            item_field.format(formatted_code, formatter)?;

            if value_pairs_iter.peek().is_some() {
                write!(formatted_code, "{} ", variant.1.span().as_str())?;
            }
        }
        if let Some(final_value) = &variants.final_value_opt {
            final_value.format(formatted_code, formatter)?;
            formatted_code.push(' ');
        } else {
            formatted_code.pop();
            formatted_code.pop();
            formatted_code.push(' ');
        }
    }

    // Handle closing brace
    ItemEnum::close_curly_brace(formatted_code, formatter)?;
    Ok(())
}

impl ItemLen for ItemEnum {
    fn get_formatted_len(&self) -> usize {
        // TODO while determininig the length we may want to format to some degree and take length.
        let str_item = &self.span().as_str().len();
        *str_item as usize
    }
}

impl CurlyBrace for ItemEnum {
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
                // Add openning brace to the next line.
                writeln!(line, "\n{}", open_brace)?;
                shape = shape.block_indent(extra_width);
            }
            _ => {
                // Add opening brace to the same line
                write!(line, " {}", open_brace)?;
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
        // If shape is becoming left-most aligned or - indent just have the defualt shape
        formatter.shape = formatter
            .shape
            .shrink_left(formatter.config.whitespace.tab_spaces)
            .unwrap_or_default();
        Ok(())
    }
}
