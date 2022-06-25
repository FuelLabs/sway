use crate::{
    config::items::ItemBraceStyle,
    fmt::{Format, FormattedCode, Formatter},
    utils::{attribute::FormatDecl, bracket::CurlyBrace},
    FormatterError,
};
use sway_parse::{token::Delimiter, ItemAbi};
use sway_types::Spanned;

impl Format for ItemAbi {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // Add enum token
        formatted_code.push_str(self.abi_token.span().as_str());
        formatted_code.push(' ');

        // Add name of the abi
        formatted_code.push_str(self.name.as_str());
        Self::open_curly_brace(formatted_code, formatter);

        // Add item fields
        // abi_items
        let mut abi_items_iter = self.abi_items.get().iter().peekable();
        while let Some(item) = abi_items_iter.next() {
            let attribute_list = item.0.attribute_list.clone();
            // add indent + format attribute if it exists
            if !attribute_list.is_empty() {
                formatted_code.push_str(&formatter.shape.indent.to_string(formatter));
                for attr in attribute_list {
                    attr.format(formatted_code, formatter);
                }
            }
            // add indent + format item
            formatted_code.push_str(&formatter.shape.indent.to_string(formatter));
            formatted_code.push_str(&format!(
                "{}{}\n",
                item.0.value.span().as_str(), // FnSignature formatting (todo!)
                item.1.span().as_str(),       // SemicolonToken
            ));

            if abi_items_iter.peek().is_some() {
                formatted_code.push('\n');
            }
        }

        // abi_defs_opt
        if let Some(abi_defs) = self.abi_defs_opt.clone() {
            let mut iter = abi_defs.get().iter().peekable();
            while let Some(item) = iter.next() {
                let attribute_list = item.attribute_list.clone();
                // add indent + format attribute if it exists
                if !attribute_list.is_empty() {
                    formatted_code.push_str(&formatter.shape.indent.to_string(formatter));
                    for attr in attribute_list {
                        attr.format(formatted_code, formatter);
                    }
                }

                // add indent + format item
                formatted_code.push_str(&formatter.shape.indent.to_string(formatter));
                item.value.format(formatted_code, formatter)?; // ItemFn formatting (todo!)
                if iter.peek().is_some() {
                    formatted_code.push('\n');
                }
            }
        }
        Self::close_curly_brace(formatted_code, formatter);
        Ok(())
    }
}

impl CurlyBrace for ItemAbi {
    fn open_curly_brace(line: &mut String, formatter: &mut Formatter) {
        let brace_style = formatter.config.items.item_brace_style;
        let extra_width = formatter.config.whitespace.tab_spaces;
        let mut shape = formatter.shape;
        let open_brace = Delimiter::Brace.as_open_char();
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add openning brace to the next line.
                line.push_str(&format!("\n{}\n", open_brace));
                shape = shape.block_indent(extra_width);
            }
            _ => {
                // Add opening brace to the same line
                line.push_str(&format!(" {}\n", open_brace));
                shape = shape.block_indent(extra_width);
            }
        }

        formatter.shape = shape;
    }
    fn close_curly_brace(line: &mut String, formatter: &mut Formatter) {
        line.push(Delimiter::Brace.as_close_char());
        // If shape is becoming left-most alligned or - indent just have the defualt shape
        formatter.shape = formatter
            .shape
            .shrink_left(formatter.config.whitespace.tab_spaces)
            .unwrap_or_default();
    }
}
