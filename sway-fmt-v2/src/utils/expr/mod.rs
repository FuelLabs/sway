use crate::fmt::*;
use std::fmt::Write;
use sway_parse::{
    token::{Delimiter, PunctKind},
    AbiCastArgs, CodeBlockContents, Expr, ExprArrayDescriptor, ExprStructField,
    ExprTupleDescriptor, MatchBranch,
};
use sway_types::Spanned;

use super::bracket::{CurlyBrace, Parenthesis, SquareBracket};

pub(crate) mod abi_cast;
pub(crate) mod asm_block;
pub(crate) mod assignable;
pub(crate) mod code_block;
pub(crate) mod collections;
pub(crate) mod conditional;
pub(crate) mod struct_field;

impl Format for Expr {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Self::Path(path) => path.format(formatted_code, formatter)?,
            Self::Literal(lit) => lit.format(formatted_code, formatter)?,
            Self::AbiCast { abi_token, args } => {
                write!(formatted_code, "{} ", abi_token.span().as_str())?;
                AbiCastArgs::open_parenthesis(formatted_code, formatter)?;
                args.clone()
                    .into_inner()
                    .format(formatted_code, formatter)?;
                AbiCastArgs::close_parenthesis(formatted_code, formatter)?;
            }
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
            Self::Tuple(tuple_descriptor) => {
                ExprTupleDescriptor::open_parenthesis(formatted_code, formatter)?;
                tuple_descriptor
                    .clone()
                    .into_inner()
                    .format(formatted_code, formatter)?;
                ExprTupleDescriptor::close_parenthesis(formatted_code, formatter)?;
            }
            Self::Parens(expr) => {
                Self::open_parenthesis(formatted_code, formatter)?;
                expr.clone()
                    .into_inner()
                    .format(formatted_code, formatter)?;
                Self::close_parenthesis(formatted_code, formatter)?;
            }
            Self::Block(code_block) => {
                CodeBlockContents::open_curly_brace(formatted_code, formatter)?;
                code_block
                    .clone()
                    .into_inner()
                    .format(formatted_code, formatter)?;
                CodeBlockContents::close_curly_brace(formatted_code, formatter)?;
            }
            Self::Array(array_descriptor) => {
                ExprArrayDescriptor::open_square_bracket(formatted_code, formatter)?;
                array_descriptor
                    .clone()
                    .into_inner()
                    .format(formatted_code, formatter)?;
                ExprArrayDescriptor::close_square_bracket(formatted_code, formatter)?;
            }
            Self::Asm(asm_block) => asm_block.format(formatted_code, formatter)?,
            Self::Return {
                return_token,
                expr_opt,
            } => {
                write!(formatted_code, "{}", return_token.span().as_str())?;
                if let Some(expr) = &expr_opt {
                    write!(formatted_code, " ")?;
                    expr.format(formatted_code, formatter)?;
                }
            }
            Self::If(if_expr) => if_expr.format(formatted_code, formatter)?,
            Self::Match {
                match_token,
                value,
                branches,
            } => {
                write!(formatted_code, "{} ", match_token.span().as_str())?;
                value.format(formatted_code, formatter)?;
                MatchBranch::open_curly_brace(formatted_code, formatter)?;
                let branches = branches.clone().into_inner();
                for match_branch in branches.iter() {
                    match_branch.format(formatted_code, formatter)?;
                }
                MatchBranch::close_curly_brace(formatted_code, formatter)?;
            }
            Self::While {
                while_token,
                condition,
                block,
            } => {
                write!(formatted_code, "{} ", while_token.span().as_str())?;
                condition.format(formatted_code, formatter)?;
                CodeBlockContents::open_curly_brace(formatted_code, formatter)?;
                block
                    .clone()
                    .into_inner()
                    .format(formatted_code, formatter)?;
                CodeBlockContents::close_curly_brace(formatted_code, formatter)?;
            }
            Self::FuncApp { func, args } => {
                func.format(formatted_code, formatter)?;
                Self::open_parenthesis(formatted_code, formatter)?;
                args.clone()
                    .into_inner()
                    .format(formatted_code, formatter)?;
                Self::close_parenthesis(formatted_code, formatter)?;
            }
            Self::Index { target, arg } => {
                target.format(formatted_code, formatter)?;
                Self::open_square_bracket(formatted_code, formatter)?;
                arg.clone().into_inner().format(formatted_code, formatter)?;
                Self::close_square_bracket(formatted_code, formatter)?;
            }
            Self::MethodCall {
                target,
                dot_token,
                name,
                contract_args_opt,
                args,
            } => {
                target.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", dot_token.span().as_str())?;
                name.format(formatted_code, formatter)?;
                if let Some(contract_args) = &contract_args_opt {
                    ExprStructField::open_curly_brace(formatted_code, formatter)?;
                    contract_args
                        .clone()
                        .into_inner()
                        .format(formatted_code, formatter)?;
                    ExprStructField::close_curly_brace(formatted_code, formatter)?;
                }
                Self::open_parenthesis(formatted_code, formatter)?;
                args.clone()
                    .into_inner()
                    .format(formatted_code, formatter)?;
                Self::close_parenthesis(formatted_code, formatter)?;
            }
            Self::FieldProjection {
                target,
                dot_token,
                name,
            } => {
                target.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", dot_token.span().as_str())?;
                name.format(formatted_code, formatter)?;
            }
            Self::TupleFieldProjection {
                target,
                dot_token,
                field: _,
                field_span,
            } => {
                target.format(formatted_code, formatter)?;
                write!(
                    formatted_code,
                    "{}{}",
                    dot_token.span().as_str(),
                    field_span.as_str(),
                )?;
            }
            Self::Ref { ref_token, expr } => {
                write!(formatted_code, "{} ", ref_token.span().as_str())?;
                expr.format(formatted_code, formatter)?;
            }
            Self::Deref { deref_token, expr } => {
                write!(formatted_code, "{} ", deref_token.span().as_str())?;
                expr.format(formatted_code, formatter)?;
            }
            Self::Not { bang_token, expr } => {
                write!(formatted_code, "{}", bang_token.span().as_str())?;
                expr.format(formatted_code, formatter)?;
            }
            Self::Mul {
                lhs,
                star_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", star_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::Div {
                lhs,
                forward_slash_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", forward_slash_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::Modulo {
                lhs,
                percent_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", percent_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::Add {
                lhs,
                add_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", add_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::Sub {
                lhs,
                sub_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", sub_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::Shl {
                lhs,
                shl_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", shl_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::Shr {
                lhs,
                shr_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", shr_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::BitAnd {
                lhs,
                ampersand_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", ampersand_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::BitXor {
                lhs,
                caret_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", caret_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::BitOr {
                lhs,
                pipe_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", pipe_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::Equal {
                lhs,
                double_eq_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", double_eq_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::NotEqual {
                lhs,
                bang_eq_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", bang_eq_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::LessThan {
                lhs,
                less_than_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", less_than_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::GreaterThan {
                lhs,
                greater_than_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", greater_than_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::LessThanEq {
                lhs,
                less_than_eq_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", less_than_eq_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::GreaterThanEq {
                lhs,
                greater_than_eq_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", greater_than_eq_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::LogicalAnd {
                lhs,
                double_ampersand_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", double_ampersand_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::LogicalOr {
                lhs,
                double_pipe_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", double_pipe_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::Reassignment {
                assignable,
                reassignment_op,
                expr,
            } => {
                assignable.format(formatted_code, formatter)?;
                reassignment_op.format(formatted_code, formatter)?;
                expr.format(formatted_code, formatter)?;
            }
        }

        Ok(())
    }
}

impl Parenthesis for Expr {
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

impl SquareBracket for Expr {
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
