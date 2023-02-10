use crate::{
    comments::write_comments,
    config::items::ItemBraceStyle,
    formatter::*,
    utils::{
        map::byte_span::{ByteSpan, LeafSpans},
        CurlyBrace,
    },
};
use std::fmt::Write;
use sway_ast::{keywords::Token, token::Delimiter, ItemAbi};
use sway_types::Spanned;

impl Format for ItemAbi {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // `abi name`
        write!(formatted_code, "{} ", self.abi_token.span().as_str())?;
        self.name.format(formatted_code, formatter)?;
        Self::open_curly_brace(formatted_code, formatter)?;

        let abi_items = self.abi_items.get();

        // add pre fn_signature comments
        let end = if abi_items.is_empty() {
            self.span().end()
        }
        // if there are existing abi items, we want to end before the hash token
        // This is OK to unwrap since we've checked is_empty() above.
        else if let Some(first_attr) = abi_items.first().unwrap().0.attribute_list.first() {
            first_attr.hash_token.span().start()
        } else {
            abi_items.first().unwrap().0.value.span().start()
        };

        write_comments(
            formatted_code,
            std::ops::Range {
                start: self.name.span().end(),
                end,
            },
            formatter,
        )?;

        let mut prev_end = None;
        // abi_items
        for (fn_signature, semicolon) in self.abi_items.get().iter() {
            if let Some(end) = prev_end {
                let range = std::ops::Range {
                    start: end,
                    end: fn_signature.value.span().start(),
                };

                write_comments(formatted_code, range, formatter)?;
            }
            // add indent + format item
            write!(
                formatted_code,
                "{}",
                formatter.shape.indent.to_string(&formatter.config)?,
            )?;
            fn_signature.format(formatted_code, formatter)?;
            writeln!(
                formatted_code,
                "{}",
                semicolon.ident().as_str() // SemicolonToken
            )?;

            prev_end = Some(fn_signature.value.span().end());
        }

        // abi_defs_opt
        if let Some(abi_defs) = self.abi_defs_opt.clone() {
            for item in abi_defs.get().iter() {
                // add indent + format item
                write!(
                    formatted_code,
                    "{}",
                    formatter.shape.indent.to_string(&formatter.config)?,
                )?;
                item.format(formatted_code, formatter)?;
            }
        }

        let last_abi_item = abi_items.last();
        let start = if let Some(last_abi_item) = last_abi_item {
            // If there are ABI items and attributes:
            // we start from the hash token of the last attribute.
            if let Some(last_attr) = last_abi_item.0.attribute_list.last() {
                last_attr.hash_token.span().start()
            }
            // If there are ABI items but no attributes:
            // we start from the last item.
            else {
                last_abi_item.0.value.span().end()
            }
        }
        // If there are no ABI items:
        // we write ALL comments in the span here.
        else {
            self.span().start()
        };

        let range = std::ops::Range {
            start,
            end: self.span().end(),
        };

        write_comments(formatted_code, range, formatter)?;

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
        formatter.shape.block_indent(&formatter.config);
        let open_brace = Delimiter::Brace.as_open_char();
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add openning brace to the next line.
                write!(line, "\n{open_brace}\n")?;
            }
            _ => {
                // Add opening brace to the same line
                writeln!(line, " {open_brace}")?;
            }
        }

        Ok(())
    }
    fn close_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // If shape is becoming left-most alligned or - indent just have the defualt shape
        formatter.shape.block_unindent(&formatter.config);
        write!(
            line,
            "{}{}",
            formatter.shape.indent.to_string(&formatter.config)?,
            Delimiter::Brace.as_close_char()
        )?;

        Ok(())
    }
}

impl LeafSpans for ItemAbi {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.abi_token.span())];
        collected_spans.push(ByteSpan::from(self.name.span()));
        collected_spans.append(&mut self.abi_items.leaf_spans());
        if let Some(abi_defs) = &self.abi_defs_opt {
            collected_spans.append(&mut abi_defs.leaf_spans());
        }
        collected_spans
    }
}
