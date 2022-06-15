use crate::{
    config::items::ItemBraceStyle,
    fmt::{Format, FormattedCode, Formatter},
    utils::bracket::CurlyBrace,
};
use sway_parse::ItemEnum;

impl Format for ItemEnum {
    fn format(&self, _formatter: &mut Formatter) -> FormattedCode {
        todo!()
    }
}

impl CurlyBrace for ItemEnum {
    fn open_curly_brace(line: &mut String, formatter: &mut Formatter) {
        let brace_style = formatter.config.items.item_brace_style;
        let mut shape = formatter.shape;
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add openning brace to the next line.
                line.push_str("\n{\n");
                shape = shape.block_indent(1);
            }
            _ => {
                // Add opening brace to the same line
                line.push_str(" {\n");
                shape = shape.block_indent(1);
            }
        }

        formatter.shape = shape;
    }

    fn close_curly_brace(line: &mut String, formatter: &mut Formatter) {
        line.push('}');
        // If shape is becoming left-most alligned or - indent just have the defualt shape
        formatter.shape = formatter.shape.shrink_left(1).unwrap_or_default();
    }
}
