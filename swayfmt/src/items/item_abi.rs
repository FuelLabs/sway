use crate::{
    comments::{rewrite_with_comments, write_comments},
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
        let last = formatted_code.len();
        // `abi name`
        write!(formatted_code, "{} ", self.abi_token.span().as_str())?;
        self.name.format(formatted_code, formatter)?;

        // ` : super_trait + super_trait`
        if let Some((colon_token, traits)) = &self.super_traits {
            write!(formatted_code, " {} ", colon_token.ident().as_str())?;
            traits.format(formatted_code, formatter)?;
        }

        Self::open_curly_brace(formatted_code, formatter)?;

        let abi_items = self.abi_items.get();

        // abi_items
        for (annotated, semicolon) in abi_items.iter() {
            // add indent + format item
            write!(
                formatted_code,
                "{}",
                formatter.shape.indent.to_string(&formatter.config)?,
            )?;
            annotated.format(formatted_code, formatter)?;
            writeln!(
                formatted_code,
                "{}",
                semicolon.ident().as_str() // SemicolonToken
            )?;
        }

        if abi_items.is_empty() {
            write_comments(
                formatted_code,
                self.abi_items.span().start()..self.abi_items.span().end(),
                formatter,
            )?;
        }

        Self::close_curly_brace(formatted_code, formatter)?;

        // abi_defs_opt
        if let Some(abi_defs) = self.abi_defs_opt.clone() {
            Self::open_curly_brace(formatted_code, formatter)?;
            for item in abi_defs.get().iter() {
                // add indent + format item
                write!(
                    formatted_code,
                    "{}",
                    formatter.shape.indent.to_string(&formatter.config)?,
                )?;
                item.format(formatted_code, formatter)?;
            }
            writeln!(formatted_code)?;

            Self::close_curly_brace(formatted_code, formatter)?;
        }

        rewrite_with_comments::<ItemAbi>(
            formatter,
            self.span(),
            self.leaf_spans(),
            formatted_code,
            last,
        )?;

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
