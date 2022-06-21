use crate::{
    config::items::ItemBraceStyle,
    fmt::{Format, FormattedCode, Formatter},
    utils::bracket::CurlyBrace,
    FormatterError,
};
use sway_parse::{attribute::Annotated, ItemAbi};
use sway_types::Spanned;

impl Format for ItemAbi {
    fn format(&self, formatter: &mut Formatter) -> FormattedCode {
        let mut formatted_code = String::new();

        // Add enum token
        formatted_code.push_str(self.abi_token.span().as_str());
        formatted_code.push(' ');

        // Add name of the abi
        formatted_code.push_str(self.name.as_str());
        formatted_code.push(' ');
        Self::open_curly_brace(&mut formatted_code, formatter);

        // Add items
        formatted_code.push_str(self.abi_items.clone().into_inner());
        let items = self.abi_items.into_inner();
        if let Some(abi_defs) = self.abi_defs_opt {
            formatted_code += abi_defs
                .into_inner()
                .into_iter()
                .map(|item| -> FormattedCode {
                    let mut buf = Annotated::format(item.attribute_list, formatter);
                    buf.push_str(item.value.format(formatter))
                })
                .collect::<Vec<String>>
                .join("\n");
        }
        Self::close_curly_brace(&mut formatted_code, formatter);

        formatted_code
    }
}

impl CurlyBrace for ItemAbi {
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
