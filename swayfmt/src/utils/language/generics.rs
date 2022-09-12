use crate::{
    formatter::*,
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
        formatter.with_shape(
            formatter.shape.with_default_code_line(),
            |formatter| -> Result<(), FormatterError> {
                // `<`
                open_angle_bracket(formatted_code)?;
                // format and add parameters
                params.format(formatted_code, formatter)?;
                // `>`
                close_angle_bracket(formatted_code)?;

                Ok(())
            },
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
        let params = self.parameters.clone().into_inner();
        formatter.with_shape(
            formatter.shape.with_default_code_line(),
            |formatter| -> Result<(), FormatterError> {
                // `<`
                open_angle_bracket(formatted_code)?;
                // format and add parameters
                params.format(formatted_code, formatter)?;
                // `>`
                close_angle_bracket(formatted_code)?;

                Ok(())
            },
        )?;

        Ok(())
    }
}
