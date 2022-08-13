use crate::{
    config::items::ItemBraceStyle,
    fmt::*,
    utils::{
        bracket::CurlyBrace,
        comments::{ByteSpan, LeafSpans},
    },
};
use std::fmt::Write;
use sway_ast::{keywords::Token, token::Delimiter, ItemTrait, Traits};
use sway_types::Spanned;

impl Format for ItemTrait {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // `pub `
        if let Some(pub_token) = &self.visibility {
            write!(formatted_code, "{} ", pub_token.span().as_str())?;
        }
        // `trait name`
        write!(
            formatted_code,
            "{} {}",
            self.trait_token.span().as_str(),
            self.name.span().as_str()
        )?;
        // `: super_trait + super_trait`
        if let Some((colon_token, traits)) = &self.super_traits {
            write!(formatted_code, "{} ", colon_token.ident().as_str())?;
            traits.format(formatted_code, formatter)?;
        }
        write!(formatted_code, " ")?;
        Self::open_curly_brace(formatted_code, formatter)?;
        for (fn_signature, semicolon_token) in self.trait_items.get() {
            write!(
                formatted_code,
                "{}",
                &formatter.shape.indent.to_string(&formatter.config)?,
            )?;
            // format `Annotated<FnSignature>`
            fn_signature.format(formatted_code, formatter)?;
            writeln!(formatted_code, "{}\n", semicolon_token.ident().as_str())?;
        }
        formatted_code.pop(); // pop last ending newline
        if let Some(trait_defs) = &self.trait_defs_opt {
            Self::open_curly_brace(formatted_code, formatter)?;
            for trait_items in trait_defs.get() {
                write!(
                    formatted_code,
                    "{}",
                    formatter.shape.indent.to_string(&formatter.config)?
                )?;
                // format `Annotated<ItemFn>`
                trait_items.format(formatted_code, formatter)?;
            }
            Self::close_curly_brace(formatted_code, formatter)?;
        }
        Self::close_curly_brace(formatted_code, formatter)?;

        Ok(())
    }
}

impl CurlyBrace for ItemTrait {
    fn open_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let brace_style = formatter.config.items.item_brace_style;
        let open_brace = Delimiter::Brace.as_open_char();
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add openning brace to the next line.
                writeln!(line, "\n{}", open_brace)?;
                formatter.shape.block_indent(&formatter.config);
            }
            _ => {
                writeln!(line, "{}", open_brace)?;
                formatter.shape.block_indent(&formatter.config);
            }
        }

        Ok(())
    }
    fn close_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        writeln!(line, "{}", Delimiter::Brace.as_close_char())?;
        formatter.shape.block_unindent(&formatter.config);
        Ok(())
    }
}

impl Format for Traits {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // prefix `PathType`
        self.prefix.format(formatted_code, formatter)?;
        // additional `PathType`s
        //
        // ` + PathType`
        for (add_token, path_type) in self.suffixes.iter() {
            write!(formatted_code, " {} ", add_token.span().as_str())?;
            path_type.format(formatted_code, formatter)?;
        }

        Ok(())
    }
}

impl LeafSpans for ItemTrait {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        if let Some(visibility) = &self.visibility {
            collected_spans.push(ByteSpan::from(visibility.span()));
        }
        collected_spans.push(ByteSpan::from(self.trait_token.span()));
        collected_spans.push(ByteSpan::from(self.name.span()));
        if let Some(super_traits) = &self.super_traits {
            collected_spans.append(&mut super_traits.leaf_spans());
        }
        collected_spans.append(&mut self.trait_items.leaf_spans());
        if let Some(trait_defs) = &self.trait_defs_opt {
            collected_spans.append(&mut trait_defs.leaf_spans());
        }
        collected_spans
    }
}

impl LeafSpans for Traits {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = self.prefix.leaf_spans();
        collected_spans.append(&mut self.suffixes.leaf_spans());
        collected_spans
    }
}
