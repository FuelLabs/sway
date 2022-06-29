use crate::{
    fmt::{Format, FormattedCode, Formatter, FormatterError},
    utils::bracket::Parenthesis,
};
use std::fmt::Write;
use sway_parse::{token::Delimiter, FnArg, FnArgs, FnSignature, ItemFn};
use sway_types::Spanned;

impl Format for ItemFn {
    fn format(
        &self,
        _formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        todo!()
    }
}

pub(crate) trait FormatSig {
    fn format(
        &self,
        formatted_code: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError>;
}

impl FormatSig for FnSignature {
    fn format(
        &self,
        formatted_code: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // `pub `
        if let Some(visibility_token) = &self.visibility {
            write!(formatted_code, "{} ", visibility_token.span().as_str())?;
        }
        // `fn ` + name
        write!(
            formatted_code,
            "{} {}",
            self.fn_token.span().as_str(),
            self.name.as_str()
        )?;
        // `<T>`
        if let Some(generics) = &self.generics.clone() {
            generics.format(formatted_code, formatter)?;
        }
        // `(`
        Self::open_parenthesis(formatted_code, formatter)?;
        // FnArgs
        match self.arguments.clone().into_inner() {
            FnArgs::Static(args) => {
                let mut buf = args
                    .value_separator_pairs
                    .iter()
                    .map(|arg| format!("{}{}", arg.0.format(formatter), arg.1.span().as_str()))
                    .collect::<Vec<String>>()
                    .join(" ");
            }
            FnArgs::NonStatic { .. } => {}
        }
        // `)`
        Self::close_parenthesis(formatted_code, formatter)?;
        Ok(())
    }
}

impl Parenthesis for FnSignature {
    fn open_parenthesis(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        line.push(Delimiter::Parenthesis.as_open_char());
        Ok(())
    }
    fn close_parenthesis(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        line.push(Delimiter::Parenthesis.as_close_char());
        Ok(())
    }
}

trait FormatFnArg {
    fn format(&self, formatter: &mut Formatter) -> String;
}

impl FormatFnArg for FnArg {
    fn format(&self, _formatter: &mut Formatter) -> String {
        let formatted_code = String::new();
        formatted_code
    }
}
