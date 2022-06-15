use crate::{
    config::items::ItemBraceStyle,
    fmt::{Format, FormattedCode, Formatter},
    utils::bracket::CurlyDelimiter,
};
use sway_parse::ItemEnum;

impl Format for ItemEnum {
    fn format(&self, _formatter: &mut Formatter) -> FormattedCode {
        todo!()
    }
}

impl CurlyDelimiter for ItemEnum {
    fn handle_open_brace(push_to: &mut String, formatter: &mut Formatter) {
        let brace_style = formatter.config.items.item_brace_style;
        let mut shape = formatter.shape;
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add openning brace to the next line.
                push_to.push_str("\n{\n");
                shape = shape.block_indent(1);
            }
            _ => {
                // Add opening brace to the same line
                push_to.push_str(" {\n");
                shape = shape.block_indent(1);
            }
        }

        formatter.shape = shape;
    }

    fn handle_closed_brace(push_to: &mut String, formatter: &mut Formatter) {
        push_to.push('}');
        // If shape is becoming left-most alligned or - indent just have the defualt shape
        formatter.shape = formatter.shape.shrink_left(1).unwrap_or_default();
    }
}
