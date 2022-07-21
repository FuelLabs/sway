use crate::{
    config::items::ItemBraceStyle,
    fmt::{Format, FormattedCode, Formatter},
    utils::{
        attribute::FormatDecl,
        bracket::CurlyBrace,
        comments::{ByteSpan, CommentVisitor},
    },
    FormatterError,
};
use std::fmt::Write;
use sway_parse::{token::Delimiter, ItemAbi};
use sway_types::Spanned;

impl Format for ItemAbi {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // `abi name`
        write!(
            formatted_code,
            "{} {}",
            self.abi_token.span().as_str(),
            self.name.as_str()
        )?;
        Self::open_curly_brace(formatted_code, formatter)?;

        // abi_items
        let mut abi_items_iter = self.abi_items.get().iter().peekable();
        while let Some(item) = abi_items_iter.next() {
            let attribute_list = item.0.attribute_list.clone();
            // add indent + format attribute if it exists
            if !attribute_list.is_empty() {
                write!(
                    formatted_code,
                    "{}",
                    &formatter.shape.indent.to_string(formatter),
                )?;
                for attr in attribute_list {
                    attr.format(formatted_code, formatter)?;
                }
            }
            // add indent + format item
            write!(
                formatted_code,
                "{}",
                &formatter.shape.indent.to_string(formatter),
            )?;
            writeln!(
                formatted_code,
                "{}{}",
                item.0.value.span().as_str(), // TODO(PR #2173): FnSignature formatting
                item.1.span().as_str()        // SemicolonToken
            )?;

            if abi_items_iter.peek().is_some() {
                writeln!(formatted_code)?;
            }
        }

        // abi_defs_opt
        if let Some(abi_defs) = self.abi_defs_opt.clone() {
            let mut iter = abi_defs.get().iter().peekable();
            while let Some(item) = iter.next() {
                let attribute_list = item.attribute_list.clone();
                // add indent + format attribute if it exists
                if !attribute_list.is_empty() {
                    write!(
                        formatted_code,
                        "{}",
                        &formatter.shape.indent.to_string(formatter),
                    )?;
                    for attr in attribute_list {
                        attr.format(formatted_code, formatter)?;
                    }
                }

                // add indent + format item
                write!(
                    formatted_code,
                    "{}",
                    &formatter.shape.indent.to_string(formatter),
                )?;
                item.value.format(formatted_code, formatter)?; // TODO(PR #2173): ItemFn formatting

                if iter.peek().is_some() {
                    writeln!(formatted_code)?;
                }
            }
        }
        Self::close_curly_brace(formatted_code, formatter)?;

        Ok(())
    }
}

impl CurlyBrace for ItemAbi {
    fn open_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let brace_style = formatter.config.items.item_brace_style;
        let extra_width = formatter.config.whitespace.tab_spaces;
        let mut shape = formatter.shape;
        let open_brace = Delimiter::Brace.as_open_char();
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add openning brace to the next line.
                write!(line, "\n{}\n", open_brace)?;
                shape = shape.block_indent(extra_width);
            }
            _ => {
                // Add opening brace to the same line
                writeln!(line, " {}", open_brace)?;
                shape = shape.block_indent(extra_width);
            }
        }

        formatter.shape = shape;
        Ok(())
    }
    fn close_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        line.push(Delimiter::Brace.as_close_char());
        // If shape is becoming left-most alligned or - indent just have the defualt shape
        formatter.shape = formatter
            .shape
            .shrink_left(formatter.config.whitespace.tab_spaces)
            .unwrap_or_default();
        Ok(())
    }
}

impl CommentVisitor for ItemAbi {
    fn collect_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.abi_token.span())];
        collected_spans.push(ByteSpan::from(self.name.span()));
        collected_spans.append(&mut self.abi_items.collect_spans());
        if let Some(abi_defs) = &self.abi_defs_opt {
            collected_spans.append(&mut abi_defs.collect_spans());
        }
        collected_spans
    }
}
