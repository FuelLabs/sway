use crate::{
    fmt::{Format, FormattedCode, Formatter},
    FormatterError,
};
use std::fmt::Write;
use sway_parse::punctuated::Punctuated;
use sway_types::Spanned;

impl<T, P> Format for Punctuated<T, P>
where
    T: Spanned,
    P: Spanned,
{
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // format and add Type & Punct
        let value_pairs = &self.value_separator_pairs;
        for pair in value_pairs.iter() {
            write!(
                formatted_code,
                "{}{} ",
                pair.0.span().as_str(),
                pair.1.span().as_str(),
            )?;
        }

        // add final value, if any
        if let Some(final_value) = &self.final_value_opt {
            formatted_code.push_str(final_value.span().as_str());
        }

        Ok(())
    }
}
