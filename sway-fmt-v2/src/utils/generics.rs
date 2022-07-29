use crate::{
    fmt::{Format, FormattedCode, Formatter, FormatterError},
};
use std::fmt::Write;
use sway_parse::{GenericArgs, GenericParams};
use sway_types::Spanned;

// In the future we will need to determine whether the generic arguments
// are better suited with a `where` clause. At present they will be
// formatted in line.
//
impl Format for GenericParams {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let params = self.parameters.clone().into_inner();

        // `<`
        write!(
            formatted_code,
            "{}",
            self.parameters.open_angle_bracket_token.span().as_str()
        )?;
        // format and add parameters
        params.format(formatted_code, formatter)?;
        // `>`
        write!(
            formatted_code,
            "{}",
            self.parameters.close_angle_bracket_token.span().as_str()
        )?;

        Ok(())
    }
}

impl Format for GenericArgs {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // Need to add `<Ty, CommaToken>` to `Punctuated::format()`
        let params = self.parameters.clone().into_inner();

        // `<`
        write!(
            formatted_code,
            "{}",
            self.parameters.open_angle_bracket_token.span().as_str()
        )?;
        // format and add parameters
        params.format(formatted_code, formatter)?;
        // `>`
        write!(
            formatted_code,
            "{}",
            self.parameters.close_angle_bracket_token.span().as_str()
        )?;

        Ok(())
    }
}
