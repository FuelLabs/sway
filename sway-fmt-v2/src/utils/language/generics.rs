use crate::{
    formatter::{shape::LineStyle, *},
    utils::{close_angle_bracket, open_angle_bracket},
};
use sway_ast::{GenericArgs, GenericParams};

impl Format for GenericParams {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let params = self.parameters.clone().into_inner();
        let prev_state = formatter.shape.code_line;
        formatter
            .shape
            .code_line
            .update_line_style(LineStyle::Normal);

        // `<`
        open_angle_bracket(formatted_code)?;
        // format and add parameters
        params.format(formatted_code, formatter)?;
        // `>`
        close_angle_bracket(formatted_code)?;

        formatter.shape.update_line_settings(prev_state);

        Ok(())
    }
}

impl Format for GenericArgs {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let params = self.parameters.clone().into_inner();
        let prev_state = formatter.shape.code_line;
        formatter
            .shape
            .code_line
            .update_line_style(LineStyle::Normal);

        // `<`
        open_angle_bracket(formatted_code)?;
        // format and add parameters
        params.format(formatted_code, formatter)?;
        // `>`
        close_angle_bracket(formatted_code)?;

        formatter.shape.update_line_settings(prev_state);

        Ok(())
    }
}
