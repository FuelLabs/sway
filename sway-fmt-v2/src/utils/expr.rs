use crate::fmt::*;
use sway_parse::Expr;

impl Format for Expr {
    fn format(
        &self,
        _formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        Ok(())
    }
}
