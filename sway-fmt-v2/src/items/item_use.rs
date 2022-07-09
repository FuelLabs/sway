use crate::{fmt::*, utils::bracket::CurlyBrace};
use std::fmt::Write;
use sway_parse::{token::Delimiter, ItemUse, UseTree};
use sway_types::Spanned;

impl Format for ItemUse {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        if let Some(pub_token) = &self.visibility {
            write!(formatted_code, "{} ", pub_token.span().as_str())?;
        }
        write!(formatted_code, "{} ", self.use_token.span().as_str())?;
        if let Some(root_import) = &self.root_import {
            write!(formatted_code, "{}", root_import.span().as_str())?;
        }
        self.tree.format(formatted_code, formatter)?;
        write!(formatted_code, "{}", self.semicolon_token.span().as_str())?;
        Ok(())
    }
}

impl Format for UseTree {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Self::Group { imports } => {
                Self::open_curly_brace(formatted_code, formatter)?;
                imports
                    .clone()
                    .into_inner()
                    .format(formatted_code, formatter)?;
                Self::close_curly_brace(formatted_code, formatter)?;
            }
            Self::Name { name } => write!(formatted_code, "{}", name.span().as_str())?,
            Self::Rename {
                name,
                as_token,
                alias,
            } => {
                write!(
                    formatted_code,
                    "{} {} {}",
                    name.span().as_str(),
                    as_token.span().as_str(),
                    alias.span().as_str()
                )?;
            }
            Self::Glob { star_token } => {
                write!(formatted_code, "{}", star_token.span().as_str())?;
            }
            Self::Path {
                prefix,
                double_colon_token,
                suffix,
            } => {
                write!(
                    formatted_code,
                    "{}{}",
                    prefix.span().as_str(),
                    double_colon_token.span().as_str()
                )?;
                suffix.format(formatted_code, formatter)?;
            }
        }
        Ok(())
    }
}

impl CurlyBrace for UseTree {
    fn open_curly_brace(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Brace.as_open_char())?;
        Ok(())
    }
    fn close_curly_brace(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Brace.as_close_char())?;
        Ok(())
    }
}
