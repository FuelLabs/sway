use crate::{
    config::items::ItemBraceStyle,
    fmt::{Format, FormattedCode, Formatter},
    utils::bracket::CurlyBrace,
};
use sway_parse::{
    token::{Delimiter, PunctKind},
    ItemStorage,
};
use sway_types::Spanned;

impl Format for ItemStorage {
    fn format(&self, formatter: &mut Formatter) -> FormattedCode {
        let mut formatted_code = String::new();

        // Add storage token
        formatted_code.push_str(self.storage_token.span().as_str());
        formatted_code.push(' ');

        // Add `{`
        Self::open_curly_brace(&mut formatted_code, formatter);

        // Get the fields
        let items = self.fields.clone().into_inner();

        // Should we apply storage field alignment

        for item in items {
            // Push the current indentation level
            formatted_code.push_str(&formatter.shape.indent.to_string(formatter));

            // Push the storage field name
            formatted_code.push_str(item.name.as_str());

            // Push the colon token
            formatted_code.push_str(item.colon_token.span().as_str());
            formatted_code.push(' ');

            // Push the ty
            formatted_code.push_str(item.ty.span().as_str());

            // Push initializer if it exists.
            if let Some(initializer) = item.initializer {
                // Push a ` `
                formatted_code.push(' ');

                let expr = initializer.1;

                // Push the `=`
                formatted_code.push(PunctKind::Equals.as_char());

                // Push a ` `
                formatted_code.push(' ');

                // Push the unformatted expr
                formatted_code.push_str(expr.span().as_str());
            }

            // TODO we are currently pushing \n directly, if we want to format storage
            // into a single line in some cases. We should handle this better!
            formatted_code.push(PunctKind::Comma.as_char());
            formatted_code.push('\n');
        }

        // Add `}`
        Self::close_curly_brace(&mut formatted_code, formatter);

        formatted_code
    }
}

impl CurlyBrace for ItemStorage {
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
