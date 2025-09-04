use crate::{
    comments::{rewrite_with_comments, write_comments},
    constants::NEW_LINE,
    formatter::*,
    utils::{
        map::byte_span::{ByteSpan, LeafSpans},
        CurlyBrace,
    },
};
use std::fmt::Write;
use sway_ast::{
    keywords::{AbiToken, ColonToken, Keyword, Token},
    ItemAbi,
};
use sway_types::{ast::Delimiter, Spanned};

#[cfg(test)]
mod tests;

impl Format for ItemAbi {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let start_len = formatted_code.len();
        // `abi name`
        write!(formatted_code, "{} ", AbiToken::AS_STR)?;
        self.name.format(formatted_code, formatter)?;

        // ` : super_trait + super_trait`
        if let Some((_colon_token, traits)) = &self.super_traits {
            write!(formatted_code, " {} ", ColonToken::AS_STR)?;
            traits.format(formatted_code, formatter)?;
        }

        Self::open_curly_brace(formatted_code, formatter)?;

        let abi_items = self.abi_items.get();

        // abi_items
        for trait_item in abi_items.iter() {
            trait_item.format(formatted_code, formatter)?;
            write!(formatted_code, "{NEW_LINE}")?;
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
                item.format(formatted_code, formatter)?;
                write!(formatted_code, "{NEW_LINE}")?;
            }
            if abi_defs.get().is_empty() {
                write!(formatted_code, "{NEW_LINE}")?;
            }
            Self::close_curly_brace(formatted_code, formatter)?;
        }

        rewrite_with_comments::<ItemAbi>(
            formatter,
            self.span(),
            self.leaf_spans(),
            formatted_code,
            start_len,
        )?;

        Ok(())
    }
}

impl CurlyBrace for ItemAbi {
    fn open_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        formatter.indent();
        let open_brace = Delimiter::Brace.as_open_char();
        // Add opening brace to the same line
        writeln!(line, " {open_brace}")?;

        Ok(())
    }
    fn close_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // If shape is becoming left-most aligned or - indent just have the default shape
        formatter.unindent();
        write!(
            line,
            "{}{}",
            formatter.indent_to_str()?,
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
