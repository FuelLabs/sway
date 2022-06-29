use crate::fmt::*;
use sway_parse::{Statement, StatementLet};

impl Format for Statement {
    fn format(
        &self,
        _formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        Ok(())
    }
}

impl Format for StatementLet {
    fn format(
        &self,
        _formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        Ok(())
    }
}
