use crate::{
    fmt::*,
    utils::{
        bracket::{Parenthesis, SquareBracket},
        comments::{ByteSpan, LeafSpans},
    },
};
use std::fmt::Write;
use sway_ast::{
    expr::asm::{AsmBlock, AsmBlockContents, AsmFinalExpr, AsmRegisterDeclaration},
    token::Delimiter,
    Instruction,
};
use sway_types::Spanned;

impl Format for AsmBlock {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(formatted_code, "{} ", self.asm_token.span().as_str())?;
        Self::open_parenthesis(formatted_code, formatter)?;
        self.registers.get().format(formatted_code, formatter)?;
        Self::close_parenthesis(formatted_code, formatter)?;
        Self::open_square_bracket(formatted_code, formatter)?;
        self.contents.get().format(formatted_code, formatter)?;
        Self::close_square_bracket(formatted_code, formatter)?;

        Ok(())
    }
}

impl Parenthesis for AsmBlock {
    fn open_parenthesis(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Parenthesis.as_open_char())?;
        Ok(())
    }
    fn close_parenthesis(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Parenthesis.as_close_char())?;
        Ok(())
    }
}

impl SquareBracket for AsmBlock {
    fn open_square_bracket(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Bracket.as_open_char())?;
        Ok(())
    }
    fn close_square_bracket(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Bracket.as_close_char())?;
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
        if let Some(final_expr) = &self.final_expr_opt {
            final_expr.format(formatted_code, formatter)?;
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

impl LeafSpans for AsmBlock {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.asm_token.span())];
        collected_spans.append(&mut self.registers.leaf_spans());
        collected_spans.append(&mut self.contents.leaf_spans());
        collected_spans
    }
}

impl LeafSpans for AsmRegisterDeclaration {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.register.span())];
        if let Some(value) = &self.value_opt {
            collected_spans.append(&mut value.leaf_spans());
        }
        collected_spans
    }
}

impl LeafSpans for AsmBlockContents {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        for instruction in &self.instructions {
            collected_spans.append(&mut instruction.leaf_spans());
        }
        collected_spans
    }
}

impl LeafSpans for Instruction {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        // Visit instructions as a whole unit, meaning we cannot insert comments inside an instruction.
        vec![ByteSpan::from(self.span())]
    }
}

impl LeafSpans for AsmFinalExpr {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.register.span())];
        if let Some(ty) = &self.ty_opt {
            collected_spans.append(&mut ty.leaf_spans());
        }
        collected_spans
    }
}
