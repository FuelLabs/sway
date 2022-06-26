use crate::{
    fmt::{Format, FormattedCode, Formatter, FormatterError},
    utils::bracket::AngleBracket,
};
use std::fmt::Write;
use sway_parse::GenericParams;
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
        Self::open_angle_bracket(self.clone(), formatted_code, formatter)?;
        // format and add parameters
        params.format(formatted_code, formatter)?;
        // `>`
        Self::close_angle_bracket(self.clone(), formatted_code, formatter)?;

        Ok(())
    }
}

impl AngleBracket for GenericParams {
    fn open_angle_bracket(
        self,
        line: &mut String,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        writeln!(
            line,
            "{}",
            self.parameters.open_angle_bracket_token.span().as_str()
        )?;
        Ok(())
    }
    fn close_angle_bracket(
        self,
        line: &mut String,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        writeln!(
            line,
            "{}",
            self.parameters.close_angle_bracket_token.span().as_str()
        )?;
        Ok(())
    }
}
