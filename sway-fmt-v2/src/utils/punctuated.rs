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
        write!(
            formatted_code,
            "{}",
            self.value_separator_pairs
                .iter()
                .map(|pair| format!("{}{}", pair.0.span().as_str(), pair.1.span().as_str()))
                .collect::<Vec<String>>()
                .join(" ")
        )?;
        formatted_code.pop(); // pop the ending comma

        // add boxed type
        if let Some(final_value) = &self.final_value_opt {
            writeln!(formatted_code, "{}", final_value.span().as_str())?;
        }

        Ok(())
    }
}
