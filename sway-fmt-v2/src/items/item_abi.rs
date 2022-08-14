use crate::{
    config::items::ItemBraceStyle,
    fmt::*,
    utils::{
        bracket::CurlyBrace,
        comments::{ByteSpan, LeafSpans},
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
        write!(
            formatted_code,
            "{} {}",
            self.abi_token.span().as_str(),
            self.name.as_str()
        )?;
        Self::open_curly_brace(formatted_code, formatter)?;

        // abi_items
        let mut abi_items_iter = self.abi_items.get().iter().peekable();
        while let Some((fn_signature, semicolon)) = abi_items_iter.next() {
            // add indent + format item
            fn_signature.format(formatted_code, formatter)?;
            writeln!(
                formatted_code,
                "{}",
                semicolon.ident().as_str() // SemicolonToken
            )?;
            if abi_items_iter.peek().is_some() {
                writeln!(formatted_code)?;
            }
        }

        // abi_defs_opt
        if let Some(abi_defs) = self.abi_defs_opt.clone() {
            let mut iter = abi_defs.get().iter().peekable();
            while let Some(item) = iter.next() {
                // add indent + format item
                item.format(formatted_code, formatter)?;
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
        let open_brace = Delimiter::Brace.as_open_char();
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add openning brace to the next line.
                write!(line, "\n{}\n", open_brace)?;
                formatter.shape.block_indent(&formatter.config);
            }
            _ => {
                // Add opening brace to the same line
                writeln!(line, " {}", open_brace)?;
                formatter.shape.block_indent(&formatter.config);
            }
        }

        Ok(())
    }
    fn close_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        line.push(Delimiter::Brace.as_close_char());
        // If shape is becoming left-most alligned or - indent just have the defualt shape
        formatter.shape.block_unindent(&formatter.config);
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
