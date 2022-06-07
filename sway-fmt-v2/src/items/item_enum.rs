use crate::fmt::{Format, FormattedCode, Formatter};
use sway_parse::ItemEnum;
use sway_types::Spanned;

impl Format for ItemEnum {
    fn format(&self, formatter: &Formatter) -> FormattedCode {
        let mut current_ident_level = 0;
        let mut formatted_code = String::new();
        let enum_variant_align_threshold = formatter.config.structures.enum_variant_align_threshold;

        if let Some(visibility_token) = &self.visibility {
            add_formatted_part(
                &mut formatted_code,
                visibility_token.span().as_str(),
                current_ident_level,
                true,
                true,
            );
        }

        add_formatted_part(
            &mut formatted_code,
            self.enum_token.span().as_str(),
            current_ident_level,
            true,
            false,
        );

        let bracket_on_new_line = formatter.config.items.item_brace_style;
        match bracket_on_new_line {
            crate::config::items::ItemBraceStyle::AlwaysNextLine => {
                // Add name of the enum.
                add_formatted_part(
                    &mut formatted_code,
                    self.name.as_str(),
                    current_ident_level,
                    false,
                    false,
                );

                // Add openning bracet to the next line.
                add_formatted_part(
                    &mut formatted_code,
                    "\n{\n",
                    current_ident_level,
                    false,
                    true,
                );
                current_ident_level += 4;
            }
            _ => {
                // Add name of the enum followed by a trailing space.
                add_formatted_part(
                    &mut formatted_code,
                    self.name.as_str(),
                    current_ident_level,
                    true,
                    false,
                );

                // Add opening bracet to the same line
                add_formatted_part(
                    &mut formatted_code,
                    "{\n",
                    current_ident_level,
                    false,
                    false,
                );
                current_ident_level += 4;
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

        // In second iteration we are going to be comparing
        for (index, type_field) in type_fields.iter().enumerate() {
            let type_field = &type_field.0;
            add_formatted_part(
                &mut formatted_code,
                type_field.name.as_str(),
                current_ident_level,
                false,
                true,
            );
            formatted_code.push(':');

            // Currently does not apply custom formatting for ty, wonder if the 'ty' is a struct it will get handled by the item_struct.rs.
            let current_variant_length = variant_length[index];

            if current_variant_length < max_valid_variant_length {
                // We need to add allignment between : and ty
                // Here we assume that difference between max_valid_variant_length
                // and current_variant_length is smaller than u32::MAX.

                // max_valid_variant_length: the length of the variant that we are taking as a reference to allign
                // current_variant_length: the length of the current variant that we are trying to format
                // + 1 is coming from the fact that: After name of the reference variant + ':'
                // a space is added so we should allign with taking that into account.
                let required_allignment = (max_valid_variant_length - current_variant_length + 1)
                    .try_into()
                    .unwrap();
                add_formatted_part(
                    &mut formatted_code,
                    type_field.ty.span().as_str(),
                    required_allignment,
                    false,
                    true,
                );
            } else {
                add_formatted_part(
                    &mut formatted_code,
                    type_field.ty.span().as_str(),
                    1,
                    false,
                    true,
                );
            }
            // Check if this is the last enum variant, if so not add the comma.
            if index != variant_length.len()-1 {
                // Here we assume that next enum variant is going to be in the new line but
                // from the config we may understand next enum variant should be in the same line instead.
                formatted_code.push(',');
            }
            formatted_code.push('\n');
        }
        formatted_code.push('}');
        formatted_code
    }
}

fn add_formatted_part(
    formatted_code: &mut String,
    formatted_part_to_add: &str,
    ident_level: u32,
    add_trailing_space: bool,
    with_ident: bool,
) {
    if with_ident {
        // Currently assumes formatter just use whitespaces, i.e no tabs!
        let starting_ident = (0..ident_level).map(|_| ' ').collect::<String>();
        formatted_code.push_str(&starting_ident);
    }
    formatted_code.push_str(formatted_part_to_add);
    if add_trailing_space {
        formatted_code.push(' ');
    }
}

//TODO(kaya) : Add tests.
