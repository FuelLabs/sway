use super::bracket::{close_angle_bracket, open_angle_bracket};
use crate::fmt::{Format, FormattedCode, Formatter, FormatterError};
use sway_parse::{GenericArgs, GenericParams};

use super::indent_style::LineStyle;

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
        let prev_state = formatter.shape.line_style;
        formatter.shape.line_style = LineStyle::Normal;

        // `<`
        open_angle_bracket(formatted_code)?;
        // format and add parameters
        params.format(formatted_code, formatter)?;
        // `>`
        close_angle_bracket(formatted_code)?;

        formatter.shape.line_style = prev_state;

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
        open_angle_bracket(formatted_code)?;
        // format and add parameters
        params.format(formatted_code, formatter)?;
        // `>`
        close_angle_bracket(formatted_code)?;

        Ok(())
    }
}
