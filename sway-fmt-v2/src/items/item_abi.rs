use crate::{
    config::items::ItemBraceStyle,
    fmt::{Format, FormattedCode, Formatter},
    utils::{attribute::FormatDecl, bracket::CurlyBrace},
};
use sway_parse::{token::Delimiter, AttributeDecl, ItemAbi};
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

        // Add item fields
        // abi_items
        formatted_code += self
            .abi_items
            .clone()
            .into_inner()
            .iter()
            .map(|item| -> FormattedCode {
                let mut buf = String::new();
                for attr in item.0.attribute_list.clone() {
                    AttributeDecl::format(&attr, &mut buf, formatter)
                }
                buf.push_str(&format!(
                    "{}{}",
                    item.0.value.span().as_str(), // FnSignature
                    item.1.span().as_str(),       // SemicolonToken
                ));

                buf
            })
            .collect::<Vec<String>>()
            .join("\n")
            .as_str();
        // abi_defs_opt
        if let Some(abi_defs) = self.abi_defs_opt.clone() {
            formatted_code += abi_defs
                .into_inner()
                .iter()
                .map(|item| -> FormattedCode {
                    let mut buf = String::new();
                    for attr in item.attribute_list.clone() {
                        AttributeDecl::format(&attr, &mut buf, formatter)
                    }
                    buf.push_str(&item.value.format(formatter)); // ItemFn formatting (todo!)

                    buf
                })
                .collect::<Vec<String>>()
                .join("\n")
                .as_str();
        }
        Self::close_curly_brace(&mut formatted_code, formatter);

        formatted_code
    }
}

impl CurlyBrace for ItemAbi {
    fn open_curly_brace(line: &mut String, formatter: &mut Formatter) {
        let brace_style = formatter.config.items.item_brace_style;
        let mut shape = formatter.shape;
        let open_brace = Delimiter::Brace.as_open_char();
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add openning brace to the next line.
                line.push_str(&format!("\n{}\n", open_brace));
                shape = shape.block_indent(1);
            }
            _ => {
                // Add opening brace to the same line
                line.push_str(&format!(" {}\n", open_brace));
                shape = shape.block_indent(1);
            }
        }

        formatter.shape = shape;
    }
    fn close_curly_brace(line: &mut String, formatter: &mut Formatter) {
        line.push(Delimiter::Brace.as_close_char());
        // If shape is becoming left-most alligned or - indent just have the defualt shape
        formatter.shape = formatter.shape.shrink_left(1).unwrap_or_default();
    }
}
