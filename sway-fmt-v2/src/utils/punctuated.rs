use crate::{
    fmt::{Format, FormattedCode, Formatter},
    FormatterError,
};
use std::fmt::Write;
use sway_parse::{keywords::CommaToken, punctuated::Punctuated, StorageField, TypeField};
use sway_types::{Ident, Spanned};

impl<T, P> Format for Punctuated<T, P>
where
    T: Format,
    P: Format,
{
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // format and add Type & Punct
        let value_pairs = &self.value_separator_pairs;

        // Later on we may want to handle instances
        // where the user wants to keep the trailing commas.
        for pair in value_pairs.iter() {
            pair.0.format(formatted_code, formatter)?;
            pair.1.format(formatted_code, formatter)?;
        }

        // add final value, if any
        if let Some(final_value) = &self.final_value_opt {
            final_value.format(formatted_code, formatter)?;
        }

        Ok(())
    }
}

impl Format for Ident {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(formatted_code, "{}", self.span().as_str())?;
        Ok(())
    }
}

impl Format for TypeField {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(
            formatted_code,
            "{}{} ",
            self.name.span().as_str(),
            self.colon_token.span().as_str(),
        )?;
        self.ty.format(formatted_code, formatter)?;
        Ok(())
    }
}

impl Format for StorageField {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(
            formatted_code,
            "{}{} {}",
            self.name.span().as_str(),
            self.colon_token.span().as_str(),
            self.ty.span().as_str()
        )?;
        Ok(())
    }
}

impl Format for CommaToken {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(formatted_code, "{}", self.span().as_str())?;
        Ok(())
    }
}
