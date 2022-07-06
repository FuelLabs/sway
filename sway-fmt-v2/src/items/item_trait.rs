use crate::{
    fmt::{Format, FormattedCode, Formatter},
    utils::bracket::CurlyBrace,
    FormatterError,
};
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
            "{} {} ",
            self.trait_token.span().as_str(),
            self.name.span().as_str()
        )?;
        // `: super_trait + super_trait`
        if let Some(super_traits) = &self.super_traits {
            formatted_code.pop(); // pop the ending space if there is a `super_trait`
            write!(formatted_code, "{} ", super_traits.0.span().as_str())?;
            super_traits.1.format(formatted_code, formatter)?;
            write!(formatted_code, " ")?; // replace ending space
        }
        Self::open_curly_brace(formatted_code, formatter)?;
        for trait_items in self.trait_items.clone().into_inner() {
            // format `Annotated<FnSignature>`
            trait_items.0.format(formatted_code, formatter)?;
            write!(formatted_code, "{}", trait_items.1.span().as_str())?;
        }
        if let Some(trait_defs) = &self.trait_defs_opt {
            Self::open_curly_brace(formatted_code, formatter)?;
            for trait_items in trait_defs.clone().into_inner() {
                // format `Annotated<ItemFn>`
                trait_items.format(formatted_code, formatter)?;
            }
            Self::close_curly_brace(formatted_code, formatter)?;
        }

        Ok(())
    }
}

impl CurlyBrace for ItemTrait {
    fn open_curly_brace(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        writeln!(line, "{}", Delimiter::Brace.as_open_char())?;
        Ok(())
    }
    fn close_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Brace.as_close_char())?;
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
        for paths in self.suffixes.iter() {
            write!(formatted_code, " {}", paths.0.span().as_str())?;
            paths.1.format(formatted_code, formatter)?;
        }

        Ok(())
    }
}
