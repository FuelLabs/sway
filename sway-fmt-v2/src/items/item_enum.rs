use crate::fmt::{Format, FormattedCode, Formatter};
use sway_parse::ItemEnum;
use sway_types::Spanned;

impl Format for ItemEnum {
    fn format(&self, formatter: &mut Formatter) -> FormattedCode {
        // TODO: creating this formatted_code with String::new() will likely cause lots of
        // reallocations maybe we can explore how we can do this, starting with with_capacity may help.
        let mut formatted_code = String::new();
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
        // Uncomment this once #1967 is addressed
        // handle_open_bracket(&mut formatted_code, formatter);

        let type_fields = &self.fields.clone().into_inner().value_separator_pairs;

        // In first iteration we are going to be collecting the lengths of the enum variants.
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
        // for calculating the alignment required.
        for (index, type_field) in type_fields.iter().enumerate() {
            let type_field = &type_field.0;
            // Push the current indentation level
            formatted_code.push_str(&formatter.shape.indent.to_string(formatter));
            formatted_code.push_str(type_field.name.as_str());
            formatted_code.push_str(" : ");

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
            // TODO: We are currently converting ty to string directly but we will probably need to format ty before adding.
            formatted_code.push_str(type_field.ty.span().as_str());
            formatted_code.push(',');

            // TODO: Here we assume that next enum variant is going to be in the new line but
            // from the config we may understand next enum variant should be in the same line instead.
            formatted_code.push('\n');
        }
        // Uncomment this once #1967 is addressed
        // handle_close_bracket(&mut formatted_code, formatter);
        formatted_code
    }
}
