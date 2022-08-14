use crate::fmt::*;
use std::fmt::Write;
use sway_ast::{ItemBreak, ItemContinue};
use sway_types::Spanned;

impl Format for ItemBreak {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        writeln!(
            formatted_code,
            "{}{}{}",
            formatter.shape.indent.to_string(&formatter.config)?,
            self.break_token.span().as_str(),
            self.semicolon_token.span().as_str()
        )?;

        Ok(())
    }
}

impl Format for ItemContinue {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        writeln!(
            formatted_code,
            "{}{}{}",
            formatter.shape.indent.to_string(&formatter.config)?,
            self.break_token.span().as_str(),
            self.semicolon_token.span().as_str()
        )?;

        Ok(())
    }
}
