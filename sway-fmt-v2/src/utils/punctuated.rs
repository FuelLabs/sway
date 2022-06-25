use crate::fmt::{Format, FormattedCode, Formatter};
use sway_parse::punctuated::Punctuated;
use sway_types::Spanned;

impl<T, P> Format for Punctuated<T, P>
where
    T: Spanned,
    P: Spanned,
{
    fn format(&self, _formatter: &mut Formatter) -> FormattedCode {
        let mut formatted_code = FormattedCode::new();

        // format and add Type & Punct
        let mut buf = self
            .value_separator_pairs
            .iter()
            .map(|pair| format!("{}{}", pair.0.span().as_str(), pair.1.span().as_str()))
            .collect::<Vec<String>>()
            .join(" ");
        buf.pop(); // pop the ending comma
        formatted_code.push_str(&buf);

        // add boxed type
        if let Some(final_value) = &self.final_value_opt {
            formatted_code.push_str(final_value.span().as_str());
        }

        formatted_code
    }
}
