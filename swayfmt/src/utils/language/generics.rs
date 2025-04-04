use crate::{
    formatter::*,
    utils::{close_angle_bracket, colon, open_angle_bracket},
};
use sway_ast::{
    generics::GenericParam,
    keywords::{ConstToken, Keyword},
    GenericArgs, GenericParams,
};

impl Format for GenericParam {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            GenericParam::Trait { ident } => ident.format(formatted_code, formatter),
            GenericParam::Const { ident, ty } => {
                use std::fmt::Write;
                write!(formatted_code, "{} ", ConstToken::AS_STR)?;
                let _ = ident.format(formatted_code, formatter);
                let _ = colon(formatted_code);
                write!(formatted_code, " ")?;
                ty.format(formatted_code, formatter)
            }
        }
    }
}

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
