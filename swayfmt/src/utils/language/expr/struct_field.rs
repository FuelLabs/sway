use crate::{
    formatter::{
        shape::{ExprKind, LineStyle},
        *,
    },
    utils::{
        language::expr::should_write_multiline,
        map::byte_span::{ByteSpan, LeafSpans},
        CurlyBrace,
    },
};
use std::fmt::Write;
use sway_ast::{
    keywords::{ColonToken, Token},
    ExprStructField,
};
use sway_types::{ast::Delimiter, Spanned};

impl Format for ExprStructField {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(formatted_code, "{}", self.field_name.as_raw_ident_str())?;
        if let Some((_colon_token, expr)) = &self.expr_opt {
            formatter.with_shape(
                formatter
                    .shape
                    .with_code_line_from(LineStyle::Inline, ExprKind::Struct),
                |formatter| -> Result<(), FormatterError> {
                    let mut expr_str = FormattedCode::new();
                    expr.format(&mut expr_str, formatter)?;

                    let expr_str = if should_write_multiline(&expr_str, formatter) {
                        let mut expr_str = FormattedCode::new();
                        formatter.shape.code_line.update_expr_new_line(true);
                        expr.format(&mut expr_str, formatter)?;
                        expr_str
                    } else {
                        expr_str
                    };
                    write!(formatted_code, "{} {}", ColonToken::AS_STR, expr_str)?;
                    Ok(())
                },
            )?;
        }

        Ok(())
    }
}

impl CurlyBrace for ExprStructField {
    fn open_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // Add opening brace to the same line
        write!(line, " {}", Delimiter::Brace.as_open_char())?;
        formatter.indent();

        Ok(())
    }

    fn close_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // Unindent by one block
        formatter.unindent();
        match formatter.shape.code_line.line_style {
            LineStyle::Inline => write!(line, "{}", Delimiter::Brace.as_close_char())?,
            _ => write!(
                line,
                "{}{}",
                formatter.indent_to_str()?,
                Delimiter::Brace.as_close_char()
            )?,
        }

        Ok(())
    }
}

impl LeafSpans for ExprStructField {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.field_name.span())];
        if let Some((colon_token, expr)) = &self.expr_opt {
            collected_spans.push(ByteSpan::from(colon_token.span()));
            collected_spans.append(&mut expr.leaf_spans());
        }
        collected_spans
    }
}
