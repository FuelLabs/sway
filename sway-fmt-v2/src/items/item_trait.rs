use crate::{config::items::ItemBraceStyle, fmt::*, utils::bracket::CurlyBrace};
use std::fmt::Write;
use sway_parse::{token::Delimiter, ItemTrait, Traits};
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
        if let Some(super_traits) = &self.super_traits {
            write!(formatted_code, "{} ", super_traits.0.span().as_str())?;
            super_traits.1.format(formatted_code, formatter)?;
        }
        write!(formatted_code, " ")?;
        Self::open_curly_brace(formatted_code, formatter)?;
        for trait_items in self.trait_items.clone().into_inner() {
            write!(
                formatted_code,
                "{}",
                formatter.shape.indent.to_string(formatter)
            )?;
            // format `Annotated<FnSignature>`
            trait_items.0.format(formatted_code, formatter)?;
            writeln!(formatted_code, "{}\n", trait_items.1.span().as_str())?;
        }
        formatted_code.pop(); // pop last ending newline
        if let Some(trait_defs) = &self.trait_defs_opt {
            Self::open_curly_brace(formatted_code, formatter)?;
            for trait_items in trait_defs.clone().into_inner() {
                write!(
                    formatted_code,
                    "{}",
                    formatter.shape.indent.to_string(formatter)
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
        let extra_width = formatter.config.whitespace.tab_spaces;
        let mut shape = formatter.shape;
        let open_brace = Delimiter::Brace.as_open_char();
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add openning brace to the next line.
                writeln!(line, "\n{}", open_brace)?;
                shape = shape.block_indent(extra_width);
            }
            _ => {
                writeln!(line, "{}", open_brace)?;
                shape = shape.block_indent(extra_width);
            }
        }

        formatter.shape = shape;
        Ok(())
    }
    fn close_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        writeln!(line, "{}", Delimiter::Brace.as_close_char())?;
        formatter.shape = formatter
            .shape
            .shrink_left(formatter.config.whitespace.tab_spaces)
            .unwrap_or_default();
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
        for paths in self.suffixes.iter() {
            write!(formatted_code, " {} ", paths.0.span().as_str())?;
            paths.1.format(formatted_code, formatter)?;
        }

        Ok(())
    }
}
