use crate::fmt::*;
use std::fmt::Write;
use sway_parse::Expr;
use sway_types::Spanned;

#[allow(unused_variables)]
// TODO:
impl Format for Expr {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(formatted_code, "{}", self.span().as_str())?;
        // match self {
        //     Self::Path(path) => {}
        //     Self::Literal(lit) => {}
        //     Self::AbiCast { abi_token, args } => {}
        //     Self::Struct { path, fields } => {}
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
        // }

        Ok(())
    }
}
