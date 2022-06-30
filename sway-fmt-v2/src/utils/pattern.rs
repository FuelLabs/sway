use crate::fmt::*;
use std::fmt::Write;
use sway_parse::{token::Delimiter, Pattern};
use sway_types::Spanned;

use super::bracket::{CurlyBrace, Parenthesis};

impl Format for Pattern {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Self::Wildcard { underscore_token } => {
                formatted_code.push_str(underscore_token.span().as_str())
            }
            Self::Var { mutable, name } => {
                if let Some(mut_token) = mutable {
                    write!(formatted_code, "{} ", mut_token.span().as_str())?;
                }
                // maybe add `Ident::format()`, not sure if needed yet.
                formatted_code.push_str(name.span().as_str());
            }
            Self::Literal(lit) => lit.format(formatted_code, formatter)?,
            Self::Constant(path) => path.format(formatted_code, formatter)?,
            Self::Constructor { path, args } => {
                path.format(formatted_code, formatter)?;
                Self::open_parenthesis(formatted_code, formatter)?;
                // need to add `<Pattern, CommaToken>` to `Punctuated::format()`
                args.clone()
                    .into_inner()
                    .format(formatted_code, formatter)?;
                Self::close_parenthesis(formatted_code, formatter)?;
            }
            Self::Struct { path, fields } => {
                path.format(formatted_code, formatter)?;
                Self::open_curly_brace(formatted_code, formatter)?;
                // need to add `<PatternStructField, CommaToken>` to `Punctuated::format()`
                fields
                    .clone()
                    .into_inner()
                    .format(formatted_code, formatter)?;
                Self::close_curly_brace(formatted_code, formatter)?;
            }
            Self::Tuple(args) => {
                Self::open_parenthesis(formatted_code, formatter)?;
                // need to add `<Pattern, CommaToken>` to `Punctuated::format()`
                args.clone()
                    .into_inner()
                    .format(formatted_code, formatter)?;
                Self::close_parenthesis(formatted_code, formatter)?;
            }
        }
        Ok(())
    }
}

// Currently these just push their respective chars, we may need to change this
impl Parenthesis for Pattern {
    fn open_parenthesis(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        line.push(Delimiter::Parenthesis.as_open_char());
        Ok(())
    }
    fn close_parenthesis(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        line.push(Delimiter::Parenthesis.as_close_char());
        Ok(())
    }
}
impl CurlyBrace for Pattern {
    fn open_curly_brace(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        line.push(Delimiter::Brace.as_open_char());
        Ok(())
    }
    fn close_curly_brace(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        line.push(Delimiter::Brace.as_close_char());
        Ok(())
    }
}
