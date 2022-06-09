use crate::{
    fmt::{Format, FormattedCode, Formatter},
    utils::indent_style::Shape,
};
use sway_parse::ItemEnum;
use sway_types::Spanned;

// current_ident_level will be replaced by incoming block ident logic.
// usage of \n will be replaced by more structed way of handling new lines.
impl Format for ItemEnum {
    fn format(&self, formatter: &Formatter, shape: &mut Shape) -> FormattedCode {
        let mut shape = *shape;
        // TODO: creating this formatted_code with String::new() will likely cause lots of
        // reallocations maybe we can explore how we can do this, starting with with_capacity may help.
        let mut formatted_code = String::new();
        let enum_variant_align_threshold = formatter.config.structures.enum_variant_align_threshold;

        if let Some(visibility_token) = &self.visibility {
            add_formatted_part(
                formatter,
                &mut formatted_code,
                visibility_token.span().as_str(),
                true,
                &shape,
            );
        }

        add_formatted_part(
            formatter,
            &mut formatted_code,
            self.enum_token.span().as_str(),
            true,
            &shape,
        );

        // Is this the relevant config option to look? What does item stand for exactly?
        let bracket_on_new_line = formatter.config.items.item_brace_style;
        match bracket_on_new_line {
            crate::config::items::ItemBraceStyle::AlwaysNextLine => {
                // Add name of the enum.
                add_formatted_part(
                    formatter,
                    &mut formatted_code,
                    self.name.as_str(),
                    false,
                    &shape,
                );

                // Add openning bracet to the next line.
                add_formatted_part(formatter, &mut formatted_code, "\n{\n", false, &shape);
                shape = shape.block_indent(1);
            }
            _ => {
                // Add name of the enum followed by a trailing space.
                add_formatted_part(
                    formatter,
                    &mut formatted_code,
                    self.name.as_str(),
                    true,
                    &shape,
                );

                // Add opening bracet to the same line
                add_formatted_part(formatter, &mut formatted_code, "{\n", false, &shape);
                shape = shape.block_indent(1);
            }
        }

        let type_fields = &self.fields.clone().into_inner().value_separator_pairs;

        // In first iteration we are going to be collecting the legnths of the enum variants.
        let variant_length: Vec<usize> = type_fields
            .iter()
            .map(|variant| variant.0.name.as_str().len())
            .collect();

        // Find the maximum length in the variant_length vector that is still smaller than enum_variant_align_threshold.
        let mut max_valid_variant_length = 0;

        variant_length.iter().for_each({
            |length| {
                if *length > max_valid_variant_length && *length < enum_variant_align_threshold {
                    max_valid_variant_length = *length;
                }
            }
        });

        // In second iteration we are going to be comparing current variants length and maximum accepted variant length
        // for calculating the allignment required.
        for (index, type_field) in type_fields.iter().enumerate() {
            let type_field = &type_field.0;
            add_formatted_part(
                formatter,
                &mut formatted_code,
                type_field.name.as_str(),
                false,
                &shape,
            );
            formatted_code.push(':');

            // Currently does not apply custom formatting for ty,
            // I am wondering if the 'ty' is a struct it will get handled by the item_struct.rs. -Kaya
            let current_variant_length = variant_length[index];

            if current_variant_length < max_valid_variant_length {
                // We need to add allignment between : and ty
                // Here we assume that difference between max_valid_variant_length
                // and current_variant_length is smaller than u32::MAX.

                // max_valid_variant_length: the length of the variant that we are taking as a reference to allign
                // current_variant_length: the length of the current variant that we are trying to format
                let required_allignment = max_valid_variant_length - current_variant_length;
                // TODO: Improve handling this
                formatted_code.push_str(&(0..required_allignment).map(|_| ' ').collect::<String>());
                add_formatted_part(
                    formatter,
                    &mut formatted_code,
                    type_field.ty.span().as_str(),
                    false,
                    &shape,
                );
            } else {
                add_formatted_part(
                    formatter,
                    &mut formatted_code,
                    type_field.ty.span().as_str(),
                    false,
                    &shape,
                );
            }
            formatted_code.push(',');
            // Here we assume that next enum variant is going to be in the new line but
            // from the config we may understand next enum variant should be in the same line instead.
            formatted_code.push('\n');
        }
        formatted_code.push('}');
        formatted_code
    }
}

fn add_formatted_part(
    formatter: &Formatter,
    formatted_code: &mut String,
    formatted_part_to_add: &str,
    add_trailing_space: bool,
    shape: &Shape,
) {
    let whitespace = &shape.indent.to_string(formatter);
    formatted_code.push_str(whitespace);
    formatted_code.push_str(formatted_part_to_add);
    if add_trailing_space {
        formatted_code.push(' ');
    }
}
