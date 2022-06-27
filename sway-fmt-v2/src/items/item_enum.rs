use crate::{
    config::items::ItemBraceStyle,
    fmt::{Format, FormattedCode, Formatter},
    utils::bracket::CurlyBrace,
    FormatterError,
};
use std::fmt::Write;
use sway_parse::{token::Delimiter, token::PunctKind, ItemEnum};
use sway_types::Spanned;

impl Format for ItemEnum {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let enum_variant_align_threshold = formatter.config.structures.enum_variant_align_threshold;

        if let Some(visibility_token) = &self.visibility {
            formatted_code.push_str(visibility_token.span().as_str());
            formatted_code.push(' ');
        }

        // Add enum token
        formatted_code.push_str(self.enum_token.span().as_str());
        formatted_code.push(' ');

        // Add name of the enum.
        formatted_code.push_str(self.name.as_str());
        Self::open_curly_brace(formatted_code, formatter)?;

        let type_fields = &self.fields.clone().into_inner().value_separator_pairs;

        // In first iteration we are going to be collecting the lengths of the enum variants.
        let variant_length: Vec<usize> = type_fields
            .iter()
            .map(|variant| variant.0.name.as_str().len())
            .collect();

        // Find the maximum length in the variant_length vector that is still smaller than enum_variant_align_threshold.
        let mut max_valid_variant_length = 0;

        variant_length.iter().for_each(|length| {
            if *length > max_valid_variant_length && *length < enum_variant_align_threshold {
                max_valid_variant_length = *length;
            }
        });

        // In second iteration we are going to be comparing current variants length and maximum accepted variant length
        // for calculating the alignment required.
        for (index, type_field) in type_fields.iter().enumerate() {
            let type_field = &type_field.0;
            // Push the current indentation level
            formatted_code.push_str(&formatter.shape.indent.to_string(formatter));
            formatted_code.push_str(type_field.name.as_str());

            // Currently does not apply custom formatting for ty,
            let current_variant_length = variant_length[index];

            if current_variant_length < max_valid_variant_length {
                // We need to add alignment between : and ty
                // max_valid_variant_length: the length of the variant that we are taking as a reference to allign
                // current_variant_length: the length of the current variant that we are trying to format
                let required_alignment = max_valid_variant_length - current_variant_length;
                // TODO: Improve handling this
                formatted_code.push_str(&(0..required_alignment).map(|_| ' ').collect::<String>());
            }
            formatted_code.push(' ');
            formatted_code.push(PunctKind::Colon.as_char());
            formatted_code.push(' ');
            // TODO: We are currently converting ty to string directly but we will probably need to format ty before adding.
            formatted_code.push_str(type_field.ty.span().as_str());
            formatted_code.push(',');

            // TODO: Here we assume that next enum variant is going to be in the new line but
            // from the config we may understand next enum variant should be in the same line instead.
            formatted_code.push('\n');
        }
        Self::close_curly_brace(formatted_code, formatter)?;
        Ok(())
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
                write!(line, "\n{}\n", open_brace)?;
                shape = shape.block_indent(extra_width);
            }
            _ => {
                // Add opening brace to the same line
                writeln!(line, " {}", open_brace)?;
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
