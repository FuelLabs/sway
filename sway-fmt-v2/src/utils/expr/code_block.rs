use crate::{fmt::*, utils::bracket::CurlyBrace};
use std::fmt::Write;
use sway_parse::CodeBlockContents;

impl Format for CodeBlockContents {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        for statement in self.statements.iter() {
            statement.format(formatted_code, formatter)?;
        }
        if let Some(final_expr) = &self.final_expr_opt {
            write!(
                formatted_code,
                "{}",
                formatter.shape.indent.to_string(formatter)
            )?;
            final_expr.format(formatted_code, formatter)?;
            writeln!(formatted_code)?;
        }

        Ok(())
    }
}

impl CurlyBrace for CodeBlockContents {
    fn open_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        Ok(())
    }
    fn close_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        Ok(())
    }
}
