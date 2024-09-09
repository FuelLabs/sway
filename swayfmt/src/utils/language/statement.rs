use crate::{
    formatter::{shape::LineStyle, *},
    utils::map::byte_span::{ByteSpan, LeafSpans},
};
use std::fmt::Write;
use sway_ast::{Expr, Parens, Punctuated, Statement, StatementLet};
use sway_types::{Span, Spanned};

impl Format for Statement {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // later we need to decide if a statement is long enough to go on next line
        format_statement(self, formatted_code, formatter)?;

        Ok(())
    }
}

/// Remove arguments from the expression if the expression is a method call if
/// the method is a simple two path call (foo.bar()). This needed because in
/// method calls of two parts they are never broke into multiple lines.
/// Arguments however can be broken into multiple lines, and that is handled
/// by `write_function_call_arguments`
fn remove_arguments_from_expr(expr: Expr) -> Expr {
    match expr {
        Expr::MethodCall {
            target,
            dot_token,
            path_seg,
            contract_args_opt,
            args,
        } => {
            let is_simple_call = matches!(*target, Expr::Path(_));
            let target = remove_arguments_from_expr(*target);
            Expr::MethodCall {
                target: Box::new(target),
                dot_token,
                path_seg,
                contract_args_opt,
                args: if is_simple_call {
                    Parens::new(
                        Punctuated {
                            value_separator_pairs: vec![],
                            final_value_opt: None,
                        },
                        Span::dummy(),
                    )
                } else {
                    args
                },
            }
        }
        Expr::FieldProjection {
            target,
            dot_token,
            name,
        } => {
            let target = remove_arguments_from_expr(*target);
            Expr::FieldProjection {
                target: Box::new(target),
                dot_token,
                name,
            }
        }
        _ => expr,
    }
}

fn format_statement(
    statement: &Statement,
    formatted_code: &mut FormattedCode,
    formatter: &mut Formatter,
) -> Result<(), FormatterError> {
    match statement {
        Statement::Let(let_stmt) => let_stmt.format(formatted_code, formatter)?,
        Statement::Item(item) => item.format(formatted_code, formatter)?,
        Statement::Expr {
            expr,
            semicolon_token_opt,
        } => {
            let mut temp_expr = FormattedCode::new();

            remove_arguments_from_expr(expr.clone()).format(&mut temp_expr, formatter)?;
            if temp_expr.len() > formatter.shape.width_heuristics.chain_width {
                let update_expr_new_line = if !matches!(
                    expr,
                    Expr::MethodCall { .. }
                        | Expr::FuncApp { func: _, args: _ }
                        | Expr::If(_)
                        | Expr::While {
                            while_token: _,
                            condition: _,
                            block: _
                        }
                ) {
                    // Method calls, If, While should not tamper with the
                    // expr_new_line because that would be inherited for all
                    // statements. That should be applied at the lowest level
                    // possible (ideally at the expression level)
                    formatter.shape.code_line.expr_new_line
                } else if formatter.shape.code_line.expr_new_line {
                    // already enabled
                    true
                } else {
                    formatter.shape.code_line.update_expr_new_line(true);
                    false
                };
                // reformat the expression adding a break
                expr.format(formatted_code, formatter)?;
                formatter
                    .shape
                    .code_line
                    .update_expr_new_line(update_expr_new_line);
            } else {
                expr.format(formatted_code, formatter)?;
            }
            if let Some(semicolon) = semicolon_token_opt {
                if formatter.shape.code_line.line_style == LineStyle::Inline {
                    write!(formatted_code, "{}", semicolon.span().as_str())?;
                } else {
                    writeln!(formatted_code, "{}", semicolon.span().as_str())?;
                }
            }
        }
        Statement::Error(_, _) => {
            return Err(FormatterError::SyntaxError);
        }
    }

    Ok(())
}

impl Format for StatementLet {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // `let `
        write!(formatted_code, "{} ", self.let_token.span().as_str())?;
        // pattern
        self.pattern.format(formatted_code, formatter)?;
        // `: Ty`
        if let Some(ty) = &self.ty_opt {
            write!(formatted_code, "{} ", ty.0.span().as_str())?;
            ty.1.format(formatted_code, formatter)?;
        }
        // ` = `
        write!(formatted_code, " {} ", self.eq_token.span().as_str())?;
        // expr
        self.expr.format(formatted_code, formatter)?;
        if formatter.shape.code_line.line_style == LineStyle::Inline {
            // `;`
            write!(formatted_code, "{}", self.semicolon_token.span().as_str())?;
        } else {
            // `;\n`
            writeln!(formatted_code, "{}", self.semicolon_token.span().as_str())?;
        }

        Ok(())
    }
}

impl LeafSpans for Statement {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        match self {
            Statement::Let(statement_let) => statement_let.leaf_spans(),
            Statement::Item(item) => item.leaf_spans(),
            Statement::Expr {
                expr,
                semicolon_token_opt,
            } => {
                let mut collected_spans = expr.leaf_spans();
                if let Some(semicolon_token) = semicolon_token_opt {
                    collected_spans.push(ByteSpan::from(semicolon_token.span()));
                }
                collected_spans
            }
            Statement::Error(spans, _) => {
                vec![sway_types::Span::join_all(spans.iter().cloned()).into()]
            }
        }
    }
}

impl LeafSpans for StatementLet {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        // Add let token's ByteSpan
        let mut collected_spans = vec![ByteSpan::from(self.let_token.span())];
        // Add pattern's ByteSpan
        collected_spans.append(&mut self.pattern.leaf_spans());
        // Add ty's ByteSpan if it exists
        if let Some(ty) = &self.ty_opt {
            collected_spans.push(ByteSpan::from(ty.0.span()));
            collected_spans.append(&mut ty.1.leaf_spans());
        }
        // Add eq token's ByteSpan
        collected_spans.push(ByteSpan::from(self.eq_token.span()));
        // Add Expr's ByteSpan
        collected_spans.append(&mut self.expr.leaf_spans());
        collected_spans.push(ByteSpan::from(self.semicolon_token.span()));
        collected_spans
    }
}
