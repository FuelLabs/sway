use crate::{
    config::items::ItemBraceStyle,
    fmt::{Format, FormattedCode, Formatter},
    utils::punctuation::CurlyDelimiter,
};
use sway_parse::ItemEnum;

impl Format for ItemEnum {
    fn format(&self, _formatter: &mut Formatter) -> FormattedCode {
        todo!()
    }
}

impl CurlyDelimiter for ItemEnum {
    fn handle_open_bracket(push_to: &mut String, formatter: &mut Formatter) {
        let bracket_on_new_line = formatter.config.items.item_brace_style;
        let mut shape = formatter.shape;
        match bracket_on_new_line {
            ItemBraceStyle::AlwaysNextLine => {
                // Add openning bracet to the next line.
                push_to.push_str("\n{\n");
                shape = shape.block_indent(1);
            }
            _ => {
                // Add opening bracet to the same line
                push_to.push_str(" {\n");
                shape = shape.block_indent(1);
            }
        }

        formatter.shape = shape;
    }

    fn handle_closed_bracket(push_to: &mut String, formatter: &mut Formatter) {
        push_to.push('}');

        // If shape is becoming left-most alligned or - indent just have the defualt shape
        formatter.shape = formatter.shape.shrink_left(1).unwrap_or_default();
    }
}
