use crate::{Parse, ParseBracket, ParseErrorKind, ParseResult, ParseToEnd, Parser, ParserConsumed};

use core::ops::ControlFlow;
use sway_ast::brackets::{Braces, Parens, SquareBrackets};
use sway_ast::expr::{ReassignmentOp, ReassignmentOpVariant};
use sway_ast::keywords::{
    AbiToken, AddEqToken, AsmToken, BreakToken, CommaToken, ConstToken, ContinueToken, DivEqToken,
    DoubleColonToken, EnumToken, EqToken, FalseToken, FnToken, IfToken, ImplToken,
    OpenAngleBracketToken, PubToken, SemicolonToken, ShlEqToken, ShrEqToken, StarEqToken,
    StorageToken, StructToken, SubEqToken, TildeToken, TraitToken, TrueToken, UseToken,
};
use sway_ast::literal::{LitBool, LitBoolType};
use sway_ast::punctuated::Punctuated;
use sway_ast::token::Delimiter;
use sway_ast::{
    AbiCastArgs, CodeBlockContents, Expr, ExprArrayDescriptor, ExprStructField,
    ExprTupleDescriptor, IfCondition, IfExpr, LitInt, Literal, MatchBranch, MatchBranchKind,
    Statement, StatementLet,
};
use sway_types::{Ident, Spanned};

mod asm;
pub mod op_code;

impl ParseToEnd for AbiCastArgs {
    fn parse_to_end<'a, 'e>(
        mut parser: Parser<'a, 'e>,
    ) -> ParseResult<(AbiCastArgs, ParserConsumed<'a>)> {
        let name = parser.parse()?;
        let comma_token = parser.parse()?;
        let address = parser.parse()?;
        match parser.check_empty() {
            Some(consumed) => {
                let abi_cast_args = AbiCastArgs {
                    name,
                    comma_token,
                    address,
                };
                Ok((abi_cast_args, consumed))
            }
            None => Err(parser.emit_error(ParseErrorKind::UnexpectedTokenAfterAbiAddress)),
        }
    }
}

impl Parse for IfExpr {
    fn parse(parser: &mut Parser) -> ParseResult<IfExpr> {
        let if_token = parser.parse()?;
        let condition = parser.parse()?;
        let then_block = parser.parse()?;
        let else_opt = match parser.take() {
            Some(else_token) => {
                let else_body = match parser.peek::<IfToken>() {
                    Some(..) => {
                        let if_expr = parser.parse()?;
                        ControlFlow::Continue(Box::new(if_expr))
                    }
                    None => {
                        let else_block = parser.parse()?;
                        ControlFlow::Break(else_block)
                    }
                };
                Some((else_token, else_body))
            }
            None => None,
        };
        Ok(IfExpr {
            if_token,
            condition,
            then_block,
            else_opt,
        })
    }
}

impl Parse for IfCondition {
    fn parse(parser: &mut Parser) -> ParseResult<IfCondition> {
        if let Some(let_token) = parser.take() {
            let lhs = parser.parse()?;
            let eq_token = parser.parse()?;
            let rhs = Box::new(parse_condition(parser)?);
            Ok(IfCondition::Let {
                let_token,
                lhs,
                eq_token,
                rhs,
            })
        } else {
            let expr = Box::new(parse_condition(parser)?);
            Ok(IfCondition::Expr(expr))
        }
    }
}

impl Parse for Expr {
    fn parse(parser: &mut Parser) -> ParseResult<Expr> {
        parse_reassignment(parser, ParseExprCtx::default())
    }
}

impl ParseToEnd for CodeBlockContents {
    fn parse_to_end<'a, 'e>(
        mut parser: Parser<'a, 'e>,
    ) -> ParseResult<(CodeBlockContents, ParserConsumed<'a>)> {
        let mut statements = Vec::new();
        let (final_expr_opt, consumed) = loop {
            if let Some(consumed) = parser.check_empty() {
                break (None, consumed);
            }
            if parser.peek::<UseToken>().is_some()
                || parser.peek::<StructToken>().is_some()
                || parser.peek::<EnumToken>().is_some()
                || parser.peek::<FnToken>().is_some()
                || parser.peek::<PubToken>().is_some()
                || parser.peek::<TraitToken>().is_some()
                || parser.peek::<ImplToken>().is_some()
                || parser.peek2::<AbiToken, Ident>().is_some()
                || parser.peek::<ConstToken>().is_some()
                || parser.peek::<BreakToken>().is_some()
                || parser.peek::<ContinueToken>().is_some()
                || matches!(
                    parser.peek2::<StorageToken, Delimiter>(),
                    Some((_, Delimiter::Brace))
                )
            {
                let item = parser.parse()?;
                let statement = Statement::Item(item);
                statements.push(statement);
                continue;
            }
            if let Some(let_token) = parser.take() {
                let pattern = parser.parse()?;
                let ty_opt = match parser.take() {
                    Some(colon_token) => {
                        let ty = parser.parse()?;
                        Some((colon_token, ty))
                    }
                    None => None,
                };
                let eq_token = parser.parse()?;
                let expr = parser.parse()?;
                let semicolon_token = parser.parse()?;
                let statement_let = StatementLet {
                    let_token,
                    pattern,
                    ty_opt,
                    eq_token,
                    expr,
                    semicolon_token,
                };
                let statement = Statement::Let(statement_let);
                statements.push(statement);
                continue;
            }
            let expr = parse_statement_expr(&mut parser)?;
            if let Some(semicolon_token) = parser.take() {
                let statement = Statement::Expr {
                    expr,
                    semicolon_token_opt: Some(semicolon_token),
                };
                statements.push(statement);
                continue;
            }
            if let Some(consumed) = parser.check_empty() {
                break (Some(Box::new(expr)), consumed);
            }
            if expr.is_control_flow() {
                let statement = Statement::Expr {
                    expr,
                    semicolon_token_opt: None,
                };
                statements.push(statement);
                continue;
            }

            return Err(parser.emit_error(ParseErrorKind::UnexpectedTokenInStatement));
        };
        let code_block_contents = CodeBlockContents {
            statements,
            final_expr_opt,
        };
        Ok((code_block_contents, consumed))
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ParseExprCtx {
    pub parsing_conditional: bool,
    pub at_start_of_statement: bool,
}

impl ParseExprCtx {
    pub fn not_statement(self) -> ParseExprCtx {
        ParseExprCtx {
            at_start_of_statement: false,
            ..self
        }
    }
}

fn parse_condition(parser: &mut Parser) -> ParseResult<Expr> {
    let ctx = ParseExprCtx {
        parsing_conditional: true,
        at_start_of_statement: false,
    };
    parse_reassignment(parser, ctx)
}

fn parse_statement_expr(parser: &mut Parser) -> ParseResult<Expr> {
    let ctx = ParseExprCtx {
        parsing_conditional: false,
        at_start_of_statement: true,
    };
    parse_reassignment(parser, ctx)
}

fn parse_reassignment(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    let expr = parse_logical_or(parser, ctx)?;
    let mut reassignment_op = None;
    if parser.peek::<AddEqToken>().is_some() {
        if let Some(add_eq_token) = parser.take::<AddEqToken>() {
            reassignment_op = Some(ReassignmentOp {
                variant: ReassignmentOpVariant::AddEquals,
                span: add_eq_token.span(),
            });
        }
    }
    if parser.peek::<SubEqToken>().is_some() {
        if let Some(sub_eq_token) = parser.take::<SubEqToken>() {
            reassignment_op = Some(ReassignmentOp {
                variant: ReassignmentOpVariant::SubEquals,
                span: sub_eq_token.span(),
            });
        }
    }
    if parser.peek::<StarEqToken>().is_some() {
        if let Some(mul_eq_token) = parser.take::<StarEqToken>() {
            reassignment_op = Some(ReassignmentOp {
                variant: ReassignmentOpVariant::MulEquals,
                span: mul_eq_token.span(),
            });
        }
    }
    if parser.peek::<DivEqToken>().is_some() {
        if let Some(div_eq_token) = parser.take::<DivEqToken>() {
            reassignment_op = Some(ReassignmentOp {
                variant: ReassignmentOpVariant::DivEquals,
                span: div_eq_token.span(),
            });
        }
    }
    if parser.peek::<ShlEqToken>().is_some() {
        if let Some(shl_eq_token) = parser.take::<ShlEqToken>() {
            reassignment_op = Some(ReassignmentOp {
                variant: ReassignmentOpVariant::ShlEquals,
                span: shl_eq_token.span(),
            });
        }
    }
    if parser.peek::<ShrEqToken>().is_some() {
        if let Some(shr_eq_token) = parser.take::<ShrEqToken>() {
            reassignment_op = Some(ReassignmentOp {
                variant: ReassignmentOpVariant::ShrEquals,
                span: shr_eq_token.span(),
            });
        }
    }
    if parser.peek::<EqToken>().is_some() {
        if let Some(eq_token) = parser.take::<EqToken>() {
            reassignment_op = Some(ReassignmentOp {
                variant: ReassignmentOpVariant::Equals,
                span: eq_token.span(),
            });
        }
    }
    if let Some(reassignment_op) = reassignment_op {
        let assignable = match expr.try_into_assignable() {
            Ok(assignable) => assignable,
            Err(expr) => {
                let span = expr.span();
                return Err(
                    parser.emit_error_with_span(ParseErrorKind::UnassignableExpression, span)
                );
            }
        };
        let expr = Box::new(parse_reassignment(parser, ctx.not_statement())?);
        return Ok(Expr::Reassignment {
            assignable,
            reassignment_op,
            expr,
        });
    }
    Ok(expr)
}

fn parse_logical_or(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    let mut expr = parse_logical_and(parser, ctx)?;
    if expr.is_control_flow() && ctx.at_start_of_statement {
        return Ok(expr);
    }
    loop {
        if let Some(double_pipe_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_logical_and(parser, ctx.not_statement())?);
            expr = Expr::LogicalOr {
                lhs,
                double_pipe_token,
                rhs,
            };
            continue;
        }
        return Ok(expr);
    }
}

fn parse_logical_and(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    let mut expr = parse_comparison(parser, ctx)?;
    if expr.is_control_flow() && ctx.at_start_of_statement {
        return Ok(expr);
    }
    loop {
        if let Some(double_ampersand_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_comparison(parser, ctx.not_statement())?);
            expr = Expr::LogicalAnd {
                lhs,
                double_ampersand_token,
                rhs,
            };
            continue;
        }
        return Ok(expr);
    }
}

fn parse_comparison(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    let expr = parse_bit_or(parser, ctx)?;
    if expr.is_control_flow() && ctx.at_start_of_statement {
        return Ok(expr);
    }
    if let Some(double_eq_token) = parser.take() {
        let lhs = Box::new(expr);
        let rhs = Box::new(parse_bit_or(parser, ctx.not_statement())?);
        return Ok(Expr::Equal {
            lhs,
            double_eq_token,
            rhs,
        });
    }
    if let Some(bang_eq_token) = parser.take() {
        let lhs = Box::new(expr);
        let rhs = Box::new(parse_bit_or(parser, ctx.not_statement())?);
        return Ok(Expr::NotEqual {
            lhs,
            bang_eq_token,
            rhs,
        });
    }
    if let Some(less_than_token) = parser.take() {
        let lhs = Box::new(expr);
        let rhs = Box::new(parse_bit_or(parser, ctx.not_statement())?);
        return Ok(Expr::LessThan {
            lhs,
            less_than_token,
            rhs,
        });
    }
    if let Some(greater_than_token) = parser.take() {
        let lhs = Box::new(expr);
        let rhs = Box::new(parse_bit_or(parser, ctx.not_statement())?);
        return Ok(Expr::GreaterThan {
            lhs,
            greater_than_token,
            rhs,
        });
    }
    if let Some(less_than_eq_token) = parser.take() {
        let lhs = Box::new(expr);
        let rhs = Box::new(parse_bit_or(parser, ctx.not_statement())?);
        return Ok(Expr::LessThanEq {
            lhs,
            less_than_eq_token,
            rhs,
        });
    }
    if let Some(greater_than_eq_token) = parser.take() {
        let lhs = Box::new(expr);
        let rhs = Box::new(parse_bit_or(parser, ctx.not_statement())?);
        return Ok(Expr::GreaterThanEq {
            lhs,
            greater_than_eq_token,
            rhs,
        });
    }
    Ok(expr)
}

fn parse_bit_or(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    let mut expr = parse_bit_xor(parser, ctx)?;
    if expr.is_control_flow() && ctx.at_start_of_statement {
        return Ok(expr);
    }
    loop {
        if let Some(pipe_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_bit_xor(parser, ctx.not_statement())?);
            expr = Expr::BitOr {
                lhs,
                pipe_token,
                rhs,
            };
            continue;
        }
        return Ok(expr);
    }
}

fn parse_bit_xor(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    let mut expr = parse_bit_and(parser, ctx)?;
    if expr.is_control_flow() && ctx.at_start_of_statement {
        return Ok(expr);
    }
    loop {
        if let Some(caret_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_bit_and(parser, ctx.not_statement())?);
            expr = Expr::BitXor {
                lhs,
                caret_token,
                rhs,
            };
            continue;
        }
        return Ok(expr);
    }
}

fn parse_bit_and(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    let mut expr = parse_shift(parser, ctx)?;
    if expr.is_control_flow() && ctx.at_start_of_statement {
        return Ok(expr);
    }
    loop {
        if let Some(ampersand_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_shift(parser, ctx.not_statement())?);
            expr = Expr::BitAnd {
                lhs,
                ampersand_token,
                rhs,
            };
            continue;
        }
        return Ok(expr);
    }
}

fn parse_shift(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    let mut expr = parse_add(parser, ctx)?;
    if expr.is_control_flow() && ctx.at_start_of_statement {
        return Ok(expr);
    }
    loop {
        if let Some(shl_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_add(parser, ctx.not_statement())?);
            expr = Expr::Shl {
                lhs,
                shl_token,
                rhs,
            };
            continue;
        }
        if let Some(shr_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_add(parser, ctx.not_statement())?);
            expr = Expr::Shr {
                lhs,
                shr_token,
                rhs,
            };
            continue;
        }
        return Ok(expr);
    }
}

fn parse_add(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    let mut expr = parse_mul(parser, ctx)?;
    if expr.is_control_flow() && ctx.at_start_of_statement {
        return Ok(expr);
    }
    loop {
        if let Some(add_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_mul(parser, ctx.not_statement())?);
            expr = Expr::Add {
                lhs,
                add_token,
                rhs,
            };
            continue;
        }
        if let Some(sub_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_mul(parser, ctx.not_statement())?);
            expr = Expr::Sub {
                lhs,
                sub_token,
                rhs,
            };
            continue;
        }
        return Ok(expr);
    }
}

fn parse_mul(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    let mut expr = parse_unary_op(parser, ctx)?;
    if expr.is_control_flow() && ctx.at_start_of_statement {
        return Ok(expr);
    }
    loop {
        if let Some(star_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_unary_op(parser, ctx.not_statement())?);
            expr = Expr::Mul {
                lhs,
                star_token,
                rhs,
            };
            continue;
        }
        if let Some(forward_slash_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_unary_op(parser, ctx.not_statement())?);
            expr = Expr::Div {
                lhs,
                forward_slash_token,
                rhs,
            };
            continue;
        }
        if let Some(percent_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_unary_op(parser, ctx.not_statement())?);
            expr = Expr::Modulo {
                lhs,
                percent_token,
                rhs,
            };
            continue;
        }
        return Ok(expr);
    }
}

fn parse_unary_op(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    if let Some(ref_token) = parser.take() {
        let expr = Box::new(parse_unary_op(parser, ctx.not_statement())?);
        return Ok(Expr::Ref { ref_token, expr });
    }
    if let Some(deref_token) = parser.take() {
        let expr = Box::new(parse_unary_op(parser, ctx.not_statement())?);
        return Ok(Expr::Deref { deref_token, expr });
    }
    if let Some(bang_token) = parser.take() {
        let expr = Box::new(parse_unary_op(parser, ctx.not_statement())?);
        return Ok(Expr::Not { bang_token, expr });
    }
    parse_projection(parser, ctx)
}

fn parse_projection(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    let mut expr = parse_func_app(parser, ctx)?;
    loop {
        if let Some(arg) = SquareBrackets::try_parse_all_inner(parser, |mut parser| {
            parser.emit_error(ParseErrorKind::UnexpectedTokenAfterArrayIndex)
        })? {
            let target = Box::new(expr);
            expr = Expr::Index { target, arg };
            continue;
        }
        if let Some(dot_token) = parser.take() {
            let target = Box::new(expr);
            if let Some(name) = parser.take() {
                if !ctx.parsing_conditional {
                    if let Some(contract_args) = Braces::try_parse(parser)? {
                        let contract_args_opt = Some(contract_args);
                        let args = Parens::parse(parser)?;
                        expr = Expr::MethodCall {
                            target,
                            dot_token,
                            name,
                            contract_args_opt,
                            args,
                        };
                        continue;
                    }
                }
                if let Some(args) = Parens::try_parse(parser)? {
                    let contract_args_opt = None;
                    expr = Expr::MethodCall {
                        target,
                        dot_token,
                        name,
                        contract_args_opt,
                        args,
                    };
                    continue;
                }
                expr = Expr::FieldProjection {
                    target,
                    dot_token,
                    name,
                };
                continue;
            }
            if let Some(lit) = parser.take() {
                let lit_int = match lit {
                    Literal::Int(lit_int) => lit_int,
                    _ => {
                        let span = lit.span();
                        return Err(parser
                            .emit_error_with_span(ParseErrorKind::InvalidLiteralFieldName, span));
                    }
                };
                let LitInt {
                    span,
                    parsed,
                    ty_opt,
                } = lit_int;
                if let Some((_, _span)) = ty_opt {
                    return Err(
                        parser.emit_error_with_span(ParseErrorKind::IntFieldWithTypeSuffix, span)
                    );
                }
                let field = parsed;
                let field_span = span;
                expr = Expr::TupleFieldProjection {
                    target,
                    dot_token,
                    field,
                    field_span,
                };
                continue;
            }
            return Err(parser.emit_error(ParseErrorKind::ExpectedFieldName));
        }
        return Ok(expr);
    }
}

fn parse_func_app(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    let mut expr = parse_atom(parser, ctx)?;
    if expr.is_control_flow() && ctx.at_start_of_statement {
        return Ok(expr);
    }
    loop {
        if let Some(args) = Parens::try_parse(parser)? {
            let func = Box::new(expr);
            expr = Expr::FuncApp { func, args };
            continue;
        }
        return Ok(expr);
    }
}

fn parse_atom(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    if let Some(code_block_inner) = Braces::try_parse(parser)? {
        return Ok(Expr::Block(code_block_inner));
    }
    if let Some(array_inner) = SquareBrackets::try_parse(parser)? {
        return Ok(Expr::Array(array_inner));
    }
    if let Some((mut parser, span)) = parser.enter_delimited(Delimiter::Parenthesis) {
        if let Some(_consumed) = parser.check_empty() {
            return Ok(Expr::Tuple(Parens::new(ExprTupleDescriptor::Nil, span)));
        }
        let head = parser.parse()?;
        if let Some(comma_token) = parser.take() {
            let (tail, _consumed) = parser.parse_to_end()?;
            let tuple = ExprTupleDescriptor::Cons {
                head,
                comma_token,
                tail,
            };
            return Ok(Expr::Tuple(Parens::new(tuple, span)));
        }
        if let Some(_consumed) = parser.check_empty() {
            return Ok(Expr::Parens(Parens::new(head, span)));
        }
        return Err(
            parser.emit_error(ParseErrorKind::ExpectedCommaOrCloseParenInTupleOrParenExpression)
        );
    }
    if parser.peek::<TrueToken>().is_some() {
        let ident = parser.parse::<TrueToken>()?;
        return Ok(Expr::Literal(Literal::Bool(LitBool {
            span: ident.span(),
            kind: LitBoolType::True,
        })));
    }
    if parser.peek::<FalseToken>().is_some() {
        let ident = parser.parse::<FalseToken>()?;
        return Ok(Expr::Literal(Literal::Bool(LitBool {
            span: ident.span(),
            kind: LitBoolType::False,
        })));
    }
    if parser.peek::<AsmToken>().is_some() {
        let asm_block = parser.parse()?;
        return Ok(Expr::Asm(asm_block));
    }
    if let Some(abi_token) = parser.take() {
        let args = parser.parse()?;
        return Ok(Expr::AbiCast { abi_token, args });
    }
    if let Some(return_token) = parser.take() {
        // TODO: how to handle this properly?
        if parser.is_empty()
            || parser.peek::<CommaToken>().is_some()
            || parser.peek::<SemicolonToken>().is_some()
        {
            return Ok(Expr::Return {
                return_token,
                expr_opt: None,
            });
        }
        let expr = parser.parse()?;
        return Ok(Expr::Return {
            return_token,
            expr_opt: Some(expr),
        });
    }
    if parser.peek::<IfToken>().is_some() {
        let if_expr = parser.parse()?;
        return Ok(Expr::If(if_expr));
    }
    if let Some(match_token) = parser.take() {
        let condition = Box::new(parse_condition(parser)?);
        let branches = parser.parse()?;
        return Ok(Expr::Match {
            match_token,
            value: condition,
            branches,
        });
    }
    if let Some(while_token) = parser.take() {
        let condition = Box::new(parse_condition(parser)?);
        let block = parser.parse()?;
        return Ok(Expr::While {
            while_token,
            condition,
            block,
        });
    }
    if parser.peek::<OpenAngleBracketToken>().is_some()
        || parser.peek::<DoubleColonToken>().is_some()
        || parser.peek::<TildeToken>().is_some()
        || parser.peek::<Ident>().is_some()
    {
        let path = parser.parse()?;
        if !ctx.parsing_conditional {
            if let Some(fields) = Braces::try_parse(parser)? {
                return Ok(Expr::Struct { path, fields });
            }
        };
        return Ok(Expr::Path(path));
    }
    if let Some(literal) = parser.take() {
        return Ok(Expr::Literal(literal));
    }
    Err(parser.emit_error(ParseErrorKind::ExpectedExpression))
}

impl Parse for ExprStructField {
    fn parse(parser: &mut Parser) -> ParseResult<ExprStructField> {
        let field_name = parser.parse()?;
        let expr_opt = match parser.take() {
            Some(colon_token) => {
                let expr = parser.parse()?;
                Some((colon_token, expr))
            }
            None => None,
        };
        Ok(ExprStructField {
            field_name,
            expr_opt,
        })
    }
}

impl ParseToEnd for ExprArrayDescriptor {
    fn parse_to_end<'a, 'e>(
        mut parser: Parser<'a, 'e>,
    ) -> ParseResult<(ExprArrayDescriptor, ParserConsumed<'a>)> {
        if let Some(consumed) = parser.check_empty() {
            let punctuated = Punctuated {
                value_separator_pairs: Vec::new(),
                final_value_opt: None,
            };
            let descriptor = ExprArrayDescriptor::Sequence(punctuated);
            return Ok((descriptor, consumed));
        }
        let value = parser.parse()?;
        if let Some(semicolon_token) = parser.take() {
            let length = parser.parse()?;
            let consumed = match parser.check_empty() {
                Some(consumed) => consumed,
                None => {
                    return Err(parser.emit_error(ParseErrorKind::UnexpectedTokenAfterArrayLength));
                }
            };
            let descriptor = ExprArrayDescriptor::Repeat {
                value: Box::new(value),
                semicolon_token,
                length,
            };
            return Ok((descriptor, consumed));
        }
        if let Some(comma_token) = parser.take() {
            let (mut punctuated, consumed): (Punctuated<_, _>, _) = parser.parse_to_end()?;
            punctuated
                .value_separator_pairs
                .insert(0, (value, comma_token));
            let descriptor = ExprArrayDescriptor::Sequence(punctuated);
            return Ok((descriptor, consumed));
        }
        if let Some(consumed) = parser.check_empty() {
            let punctuated = Punctuated {
                value_separator_pairs: Vec::new(),
                final_value_opt: Some(Box::new(value)),
            };
            let descriptor = ExprArrayDescriptor::Sequence(punctuated);
            return Ok((descriptor, consumed));
        }
        Err(parser.emit_error(ParseErrorKind::ExpectedCommaSemicolonOrCloseBracketInArray))
    }
}

impl Parse for MatchBranch {
    fn parse(parser: &mut Parser) -> ParseResult<MatchBranch> {
        let pattern = parser.parse()?;
        let fat_right_arrow_token = parser.parse()?;
        let kind = parser.parse()?;
        Ok(MatchBranch {
            pattern,
            fat_right_arrow_token,
            kind,
        })
    }
}

impl Parse for MatchBranchKind {
    fn parse(parser: &mut Parser) -> ParseResult<MatchBranchKind> {
        if let Some(block) = Braces::try_parse(parser)? {
            let comma_token_opt = parser.take();
            return Ok(MatchBranchKind::Block {
                block,
                comma_token_opt,
            });
        }
        let expr = parser.parse()?;
        let comma_token = parser.parse()?;
        Ok(MatchBranchKind::Expr { expr, comma_token })
    }
}
