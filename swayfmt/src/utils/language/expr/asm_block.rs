use crate::{
    comments::rewrite_with_comments,
    formatter::{shape::LineStyle, *},
    utils::{
        map::byte_span::{ByteSpan, LeafSpans},
        CurlyBrace, Parenthesis,
    },
};
use std::fmt::Write;
use sway_ast::{
    expr::asm::{AsmBlock, AsmBlockContents, AsmFinalExpr, AsmRegisterDeclaration},
    keywords::{AsmToken, ColonToken, Keyword, SemicolonToken, Token},
    Instruction,
};
use sway_types::{ast::Delimiter, Spanned};

impl Format for AsmBlock {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // Required for comment formatting
        let start_len = formatted_code.len();

        formatter.with_shape(formatter.shape, |formatter| -> Result<(), FormatterError> {
            let contents = self.contents.get();
            if contents.instructions.is_empty() && contents.final_expr_opt.is_some() {
                formatter
                    .shape
                    .code_line
                    .update_line_style(LineStyle::Inline)
            } else {
                formatter
                    .shape
                    .code_line
                    .update_line_style(LineStyle::Multiline)
            }
            format_asm_block(self, formatted_code, formatter)?;

            Ok(())
        })?;
        rewrite_with_comments::<AsmBlock>(
            formatter,
            self.span(),
            self.leaf_spans(),
            formatted_code,
            start_len,
        )?;

        Ok(())
    }
}

fn format_asm_block(
    asm_block: &AsmBlock,
    formatted_code: &mut FormattedCode,
    formatter: &mut Formatter,
) -> Result<(), FormatterError> {
    write!(formatted_code, "{}", AsmToken::AS_STR)?;

    formatter.with_shape(
        formatter.shape.with_default_code_line(),
        |formatter| -> Result<(), FormatterError> {
            formatter
                .shape
                .code_line
                .update_line_style(LineStyle::Normal);

            let mut inline_arguments = FormattedCode::new();
            asm_block
                .registers
                .get()
                .format(&mut inline_arguments, formatter)?;

            formatter.shape.code_line.update_expr_new_line(false);
            formatter
                .shape
                .code_line
                .update_expr_kind(shape::ExprKind::Function);
            if inline_arguments.len() > formatter.shape.width_heuristics.fn_call_width {
                formatter
                    .shape
                    .code_line
                    .update_line_style(LineStyle::Multiline);
                AsmBlock::open_parenthesis(formatted_code, formatter)?;
                formatter.indent();
                asm_block
                    .registers
                    .get()
                    .format(formatted_code, formatter)?;
                formatter.unindent();
                write!(formatted_code, "{}", formatter.indent_to_str()?)?;
                AsmBlock::close_parenthesis(formatted_code, formatter)?;
            } else {
                AsmBlock::open_parenthesis(formatted_code, formatter)?;
                write!(formatted_code, "{inline_arguments}")?;
                AsmBlock::close_parenthesis(formatted_code, formatter)?;
            }

            formatter
                .shape
                .code_line
                .update_line_style(LineStyle::Multiline);
            AsmBlock::open_curly_brace(formatted_code, formatter)?;
            asm_block.contents.get().format(formatted_code, formatter)?;
            AsmBlock::close_curly_brace(formatted_code, formatter)?;

            Ok(())
        },
    )?;

    Ok(())
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

impl CurlyBrace for AsmBlock {
    fn open_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        formatter.indent();
        match formatter.shape.code_line.line_style {
            LineStyle::Inline => {
                write!(line, " {} ", Delimiter::Brace.as_open_char())?;
            }
            _ => {
                writeln!(line, " {}", Delimiter::Brace.as_open_char())?;
            }
        }
        Ok(())
    }
    fn close_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        formatter.unindent();
        match formatter.shape.code_line.line_style {
            LineStyle::Inline => {
                write!(line, " {}", Delimiter::Brace.as_close_char())?;
            }
            _ => {
                write!(
                    line,
                    "{}{}",
                    formatter.indent_to_str()?,
                    Delimiter::Brace.as_close_char()
                )?;
            }
        }
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
        if let Some((_colon_token, expr)) = &self.value_opt {
            write!(formatted_code, "{} ", ColonToken::AS_STR)?;
            expr.format(formatted_code, formatter)?;
        }

        Ok(())
    }
}

impl Format for Instruction {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(formatted_code, "{}", self.op_code_as_str())?;
        for arg in self.register_arg_idents() {
            write!(formatted_code, " {}", arg.as_str())?
        }
        for imm in self.immediate_idents() {
            write!(formatted_code, " {}", imm.as_str())?
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
        for (instruction, _semicolon_token) in self.instructions.iter() {
            write!(formatted_code, "{}", formatter.indent_to_str()?)?;
            instruction.format(formatted_code, formatter)?;
            writeln!(formatted_code, "{}", SemicolonToken::AS_STR)?
        }
        if let Some(final_expr) = &self.final_expr_opt {
            if formatter.shape.code_line.line_style == LineStyle::Multiline {
                write!(formatted_code, "{}", formatter.indent_to_str()?)?;
                final_expr.format(formatted_code, formatter)?;
                writeln!(formatted_code)?;
            } else {
                final_expr.format(formatted_code, formatter)?;
            }
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
        if let Some((_colon_token, ty)) = &self.ty_opt {
            write!(formatted_code, "{} ", ColonToken::AS_STR)?;
            ty.format(formatted_code, formatter)?;
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
        if let Some(final_expr) = &self.final_expr_opt {
            collected_spans.append(&mut final_expr.leaf_spans());
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
