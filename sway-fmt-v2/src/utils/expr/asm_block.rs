use crate::{
    fmt::*,
    utils::bracket::{Parenthesis, SquareBracket},
};
use std::fmt::Write;
use sway_parse::expr::asm::{AsmBlock, AsmBlockContents, AsmFinalExpr, AsmRegisterDeclaration};
use sway_types::Spanned;

impl Format for AsmBlock {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(formatted_code, "{} ", self.asm_token.span().as_str())?;
        Self::open_parenthesis(formatted_code, formatter)?;
        self.registers
            .clone()
            .into_inner()
            .format(formatted_code, formatter)?;
        Self::close_parenthesis(formatted_code, formatter)?;
        Self::open_square_bracket(formatted_code, formatter)?;
        self.contents
            .clone()
            .into_inner()
            .format(formatted_code, formatter)?;
        Self::close_square_bracket(formatted_code, formatter)?;

        Ok(())
    }
}

impl Parenthesis for AsmBlock {
    fn open_parenthesis(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        Ok(())
    }
    fn close_parenthesis(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        Ok(())
    }
}

impl SquareBracket for AsmBlock {
    fn open_square_bracket(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        Ok(())
    }
    fn close_square_bracket(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        Ok(())
    }
}

impl Format for AsmRegisterDeclaration {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        self.register.format(formatted_code, formatter)?;
        if let Some(value) = &self.value_opt {
            write!(formatted_code, "{} ", value.0.span().as_str())?;
            value.1.format(formatted_code, formatter)?;
        }

        Ok(())
    }
}

impl Format for AsmBlockContents {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        for pair in self.instructions.iter() {
            writeln!(
                formatted_code,
                "{}{}",
                pair.0.span().as_str(),
                pair.1.span().as_str()
            )?;
        }

        Ok(())
    }
}

impl Format for AsmFinalExpr {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        self.register.format(formatted_code, formatter)?;
        if let Some(value) = &self.ty_opt {
            write!(formatted_code, "{} ", value.0.span().as_str())?;
            value.1.format(formatted_code, formatter)?;
        }

        Ok(())
    }
}
