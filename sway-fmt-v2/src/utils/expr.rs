use crate::{config::items::ItemBraceStyle, fmt::*};
use std::fmt::Write;
use sway_parse::{
    token::{Delimiter, PunctKind},
    Expr, ExprStructField,
};
use sway_types::Spanned;

use super::bracket::CurlyBrace;

// TODO:
impl Format for Expr {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            //     Self::Path(path) => {}
            //     Self::Literal(lit) => {}
            //     Self::AbiCast { abi_token, args } => {}
            Self::Struct { path, fields } => {
                path.format(formatted_code, formatter)?;
                ExprStructField::open_curly_brace(formatted_code, formatter)?;
                writeln!(formatted_code)?;
                let fields = fields.clone().into_inner();
                let mut value_pairs_iter = fields.value_separator_pairs.iter().peekable();
                for field in value_pairs_iter.clone() {
                    // TypeField
                    field.0.format(formatted_code, formatter)?;

                    if value_pairs_iter.peek().is_some() {
                        writeln!(formatted_code, "{}", field.1.span().as_str())?;
                    }
                }
                if let Some(final_value) = &fields.final_value_opt {
                    write!(
                        formatted_code,
                        "{}",
                        &formatter.shape.indent.to_string(formatter)
                    )?;
                    final_value.format(formatted_code, formatter)?;
                    writeln!(formatted_code, "{}", PunctKind::Comma.as_char())?;
                }
                ExprStructField::close_curly_brace(formatted_code, formatter)?;
            }
            //     Self::Tuple(tuple_descriptor) => {}
            //     Self::Parens(expr) => {}
            //     Self::Block(code_block) => {}
            //     Self::Array(array_descriptor) => {}
            //     Self::Asm(asm_block) => {}
            //     Self::Return {
            //         return_token,
            //         expr_opt,
            //     } => {}
            //     Self::If(if_expr) => {}
            //     Self::Match {
            //         match_token,
            //         value,
            //         branches,
            //     } => {}
            //     Self::While {
            //         while_token,
            //         condition,
            //         block,
            //     } => {}
            //     Self::FuncApp { func, args } => {}
            //     Self::Index { target, arg } => {}
            //     Self::MethodCall {
            //         target,
            //         dot_token,
            //         name,
            //         contract_args_opt,
            //         args,
            //     } => {}
            //     Self::FieldProjection {
            //         target,
            //         dot_token,
            //         name,
            //     } => {}
            //     Self::TupleFieldProjection {
            //         target,
            //         dot_token,
            //         field,
            //         field_span,
            //     } => {}
            //     Self::Ref { ref_token, expr } => {}
            //     Self::Deref { deref_token, expr } => {}
            //     Self::Not { bang_token, expr } => {}
            //     Self::Mul {
            //         lhs,
            //         star_token,
            //         rhs,
            //     } => {}
            //     Self::Div {
            //         lhs,
            //         forward_slash_token,
            //         rhs,
            //     } => {}
            //     Self::Modulo {
            //         lhs,
            //         percent_token,
            //         rhs,
            //     } => {}
            //     Self::Add {
            //         lhs,
            //         add_token,
            //         rhs,
            //     } => {}
            //     Self::Sub {
            //         lhs,
            //         sub_token,
            //         rhs,
            //     } => {}
            //     Self::Shl {
            //         lhs,
            //         shl_token,
            //         rhs,
            //     } => {}
            //     Self::Shr {
            //         lhs,
            //         shr_token,
            //         rhs,
            //     } => {}
            //     Self::BitAnd {
            //         lhs,
            //         ampersand_token,
            //         rhs,
            //     } => {}
            //     Self::BitXor {
            //         lhs,
            //         caret_token,
            //         rhs,
            //     } => {}
            //     Self::BitOr {
            //         lhs,
            //         pipe_token,
            //         rhs,
            //     } => {}
            //     Self::Equal {
            //         lhs,
            //         double_eq_token,
            //         rhs,
            //     } => {}
            //     Self::NotEqual {
            //         lhs,
            //         bang_eq_token,
            //         rhs,
            //     } => {}
            //     Self::LessThan {
            //         lhs,
            //         less_than_token,
            //         rhs,
            //     } => {}
            //     Self::GreaterThan {
            //         lhs,
            //         greater_than_token,
            //         rhs,
            //     } => {}
            //     Self::LessThanEq {
            //         lhs,
            //         less_than_eq_token,
            //         rhs,
            //     } => {}
            //     Self::GreaterThanEq {
            //         lhs,
            //         greater_than_eq_token,
            //         rhs,
            //     } => {}
            //     Self::LogicalAnd {
            //         lhs,
            //         double_ampersand_token,
            //         rhs,
            //     } => {}
            //     Self::LogicalOr {
            //         lhs,
            //         double_pipe_token,
            //         rhs,
            //     } => {}
            //     Self::Reassignment {
            //         assignable,
            //         reassignment_op,
            //         expr,
            //     } => {}
            _ => write!(formatted_code, "{}", self.span().as_str())?,
        }

        Ok(())
    }
}

impl Format for ExprStructField {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(
            formatted_code,
            "{}{}",
            formatter.shape.indent.to_string(formatter),
            self.field_name.span().as_str()
        )?;
        if let Some(expr) = &self.expr_opt {
            write!(formatted_code, "{} ", expr.0.span().as_str())?;
            expr.1.format(formatted_code, formatter)?;
        }

        Ok(())
    }
}

impl CurlyBrace for ExprStructField {
    fn open_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let brace_style = formatter.config.items.item_brace_style;
        let extra_width = formatter.config.whitespace.tab_spaces;
        let mut shape = formatter.shape;
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add openning brace to the next line.
                write!(line, "\n{}", Delimiter::Brace.as_open_char())?;
                shape = shape.block_indent(extra_width);
            }
            _ => {
                // Add opening brace to the same line
                write!(line, " {}", Delimiter::Brace.as_open_char())?;
                shape = shape.block_indent(extra_width);
            }
        }

        formatter.shape = shape;
        Ok(())
    }

    fn close_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // Unindent by one block
        formatter.shape.indent = formatter.shape.indent.block_unindent(formatter);
        write!(
            line,
            "{}{}",
            formatter.shape.indent.to_string(formatter),
            Delimiter::Brace.as_close_char()
        )?;
        Ok(())
    }
}
