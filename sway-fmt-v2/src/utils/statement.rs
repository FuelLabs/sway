use crate::fmt::*;
use std::fmt::Write;
use sway_parse::{Statement, StatementLet};
use sway_types::Spanned;

impl Format for Statement {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Self::Let(let_stmt) => let_stmt.format(formatted_code, formatter)?,
            Self::Item(item) => item.format(formatted_code, formatter)?,
            Self::Expr {
                expr,
                semicolon_token_opt,
            } => {
                expr.format(formatted_code, formatter)?;
                if let Some(semicolon) = semicolon_token_opt {
                    formatted_code.push_str(semicolon.span().as_str())
                }
            }
        }
        Ok(())
    }
}

impl Format for StatementLet {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // Add indent level + `let `
        write!(
            formatted_code,
            "{}{} ",
            formatter.shape.indent.to_string(formatter),
            self.let_token.span().as_str()
        )?;
        // pattern
        self.pattern.format(formatted_code, formatter)?;
        // `: Ty`
        if let Some(ty) = &self.ty_opt {
            write!(
                formatted_code,
                "{} {}",
                ty.0.span().as_str(),
                ty.1.span().as_str(), // update this when `Ty` formatting is merged
            )?;
        }
        // ` = `
        write!(formatted_code, " {} ", self.eq_token.span().as_str())?;
        // expr
        self.expr.format(formatted_code, formatter)?;
        // `;\n`
        writeln!(formatted_code, "{}", self.semicolon_token.span().as_str())?;

        Ok(())
    }
}
