use crate::{Parse, ParseBracket, ParseResult, ParseToEnd, Parser, ParserConsumed, Peek};

use sway_ast::brackets::{Braces, Parens, SquareBrackets};
use sway_ast::expr::{LoopControlFlow, ReassignmentOp, ReassignmentOpVariant};
use sway_ast::keywords::{
    AbiToken, AddEqToken, AmpersandToken, AsmToken, CommaToken, ConfigurableToken, ConstToken,
    DivEqToken, DoubleColonToken, EnumToken, EqToken, FalseToken, FnToken, HashToken, IfToken,
    ImplToken, LetToken, MutToken, OpenAngleBracketToken, PubToken, SemicolonToken, ShlEqToken,
    ShrEqToken, StarEqToken, StorageToken, StructToken, SubEqToken, TraitToken, TrueToken,
    TypeToken, UseToken,
};
use sway_ast::literal::{LitBool, LitBoolType};
use sway_ast::punctuated::Punctuated;
use sway_ast::token::DocComment;
use sway_ast::{
    AbiCastArgs, CodeBlockContents, Expr, ExprArrayDescriptor, ExprStructField,
    ExprTupleDescriptor, GenericArgs, IfCondition, IfExpr, LitInt, Literal, MatchBranch,
    MatchBranchKind, PathExpr, PathExprSegment, Statement, StatementLet,
};
use sway_error::parser_error::ParseErrorKind;
use sway_types::{ast::Delimiter, Ident, Span, Spanned};

mod asm;
pub mod op_code;

impl ParseToEnd for AbiCastArgs {
    fn parse_to_end<'a, 'e>(
        mut parser: Parser<'a, '_>,
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
                let else_body = match parser.guarded_parse::<IfToken, _>()? {
                    Some(if_expr) => LoopControlFlow::Continue(Box::new(if_expr)),
                    None => LoopControlFlow::Break(parser.parse()?),
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

impl Parse for StatementLet {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        let let_token: LetToken = parser.parse()?;

        if parser.peek::<EqToken>().is_some() {
            return Err(parser.emit_error_with_span(
                ParseErrorKind::ExpectedPattern,
                let_token
                    .span()
                    .next_char_utf8()
                    .unwrap_or_else(|| let_token.span()),
            ));
        }
        let pattern = parser.try_parse(true)?;

        let ty_opt = match parser.take() {
            Some(colon_token) => Some((colon_token, parser.parse()?)),
            None => None,
        };
        let eq_token: EqToken = parser.try_parse(true)?;
        let expr = parser.try_parse(true)?;

        // Recover on missing semicolon.
        let semicolon_token = parser.try_parse(true)?;

        Ok(StatementLet {
            let_token,
            pattern,
            ty_opt,
            eq_token,
            expr,
            semicolon_token,
        })
    }
}

impl ParseToEnd for CodeBlockContents {
    fn parse_to_end<'a, 'e>(
        mut parser: Parser<'a, '_>,
    ) -> ParseResult<(CodeBlockContents, ParserConsumed<'a>)> {
        let mut statements = Vec::new();

        let (final_expr_opt, consumed) = loop {
            if let Some(consumed) = parser.check_empty() {
                break (None, consumed);
            }

            match parser.call_parsing_function_with_recovery(parse_stmt) {
                Ok(StmtOrTail::Stmt(s)) => statements.push(s),
                Ok(StmtOrTail::Tail(e, c)) => break (Some(e), c),
                Err(r) => {
                    let (spans, error) = r
                        .recover_at_next_line_with_fallback_error(ParseErrorKind::InvalidStatement);
                    statements.push(Statement::Error(spans, error));
                }
            }
        };

        let code_block_contents = CodeBlockContents {
            statements,
            final_expr_opt,
            span: parser.full_span().clone(),
        };

        Ok((code_block_contents, consumed))
    }
}

/// A statement or a tail expression in a block.
#[allow(clippy::large_enum_variant)]
enum StmtOrTail<'a> {
    /// A statement.
    Stmt(Statement),
    /// Tail expression in a block.
    Tail(Box<Expr>, ParserConsumed<'a>),
}

/// Parses either a statement or a tail expression.
fn parse_stmt<'a>(parser: &mut Parser<'a, '_>) -> ParseResult<StmtOrTail<'a>> {
    let stmt = |s| Ok(StmtOrTail::Stmt(s));

    // Try parsing an item as a statement.
    if parser.peek::<UseToken>().is_some()
        || parser.peek::<StructToken>().is_some()
        || parser.peek::<EnumToken>().is_some()
        || parser.peek::<FnToken>().is_some()
        || parser.peek::<PubToken>().is_some()
        || parser.peek::<TraitToken>().is_some()
        || parser.peek::<ImplToken>().is_some()
        || parser.peek::<(AbiToken, Ident)>().is_some()
        || parser.peek::<ConstToken>().is_some()
        || parser.peek::<TypeToken>().is_some()
        || parser.peek::<DocComment>().is_some()
        || parser.peek::<HashToken>().is_some()
        || matches!(
            parser.peek::<(StorageToken, Delimiter)>(),
            Some((_, Delimiter::Brace))
        )
        || matches!(
            parser.peek::<(ConfigurableToken, Delimiter)>(),
            Some((_, Delimiter::Brace))
        )
    {
        return stmt(Statement::Item(parser.parse()?));
    }

    // Try a `let` statement.
    if let Some(item) = parser.guarded_parse::<LetToken, StatementLet>()? {
        return stmt(Statement::Let(item));
    }

    // Try an `expr;` statement.
    let expr = parse_statement_expr(parser)?;
    if let Some(semicolon_token) = parser.take() {
        return stmt(Statement::Expr {
            expr,
            semicolon_token_opt: Some(semicolon_token),
        });
    }

    // Reached EOF? Then an expression is a statement.
    if let Some(consumed) = parser.check_empty() {
        return Ok(StmtOrTail::Tail(Box::new(expr), consumed));
    }

    // For statements like `if`,
    // they don't need to be terminated by `;` to be statements.
    if expr.is_control_flow() {
        return stmt(Statement::Expr {
            expr,
            semicolon_token_opt: None,
        });
    }

    Err(parser.emit_error(ParseErrorKind::UnexpectedTokenInStatement))
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

/// Eats a `ReassignmentOp`, if any, from `parser`.
fn take_reassignment_op(parser: &mut Parser) -> Option<ReassignmentOp> {
    let (variant, span) = if let Some(add_eq_token) = parser.take::<AddEqToken>() {
        (ReassignmentOpVariant::AddEquals, add_eq_token.span())
    } else if let Some(sub_eq_token) = parser.take::<SubEqToken>() {
        (ReassignmentOpVariant::SubEquals, sub_eq_token.span())
    } else if let Some(mul_eq_token) = parser.take::<StarEqToken>() {
        (ReassignmentOpVariant::MulEquals, mul_eq_token.span())
    } else if let Some(div_eq_token) = parser.take::<DivEqToken>() {
        (ReassignmentOpVariant::DivEquals, div_eq_token.span())
    } else if let Some(shl_eq_token) = parser.take::<ShlEqToken>() {
        (ReassignmentOpVariant::ShlEquals, shl_eq_token.span())
    } else if let Some(shr_eq_token) = parser.take::<ShrEqToken>() {
        (ReassignmentOpVariant::ShrEquals, shr_eq_token.span())
    } else if let Some(eq_token) = parser.take::<EqToken>() {
        (ReassignmentOpVariant::Equals, eq_token.span())
    } else {
        return None;
    };
    Some(ReassignmentOp { variant, span })
}

fn parse_reassignment(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    let expr = parse_logical_or(parser, ctx)?;
    let expr_span = expr.span();

    if let Some(reassignment_op) = take_reassignment_op(parser) {
        let assignable = match expr.try_into_assignable() {
            Ok(assignable) => assignable,
            Err(expr) => {
                let span = expr.span();
                return Err(parser.emit_error_with_span(
                    ParseErrorKind::UnassignableExpression {
                        erroneous_expression_kind: expr.friendly_name(),
                        erroneous_expression_span: span,
                    },
                    expr_span,
                ));
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

fn parse_op_rhs<O: Peek>(
    parser: &mut Parser,
    ctx: ParseExprCtx,
    sub: impl Fn(&mut Parser, ParseExprCtx) -> ParseResult<Expr>,
) -> ParseResult<Option<(O, Box<Expr>)>> {
    if let Some(op_token) = parser.take() {
        let rhs = Box::new(sub(parser, ctx.not_statement())?);
        return Ok(Some((op_token, rhs)));
    }
    Ok(None)
}

fn parse_binary<O: Peek>(
    parser: &mut Parser,
    ctx: ParseExprCtx,
    sub: impl Fn(&mut Parser, ParseExprCtx) -> ParseResult<Expr>,
    combine: impl Fn(Box<Expr>, Box<Expr>, O) -> Expr,
) -> ParseResult<Expr> {
    let mut expr = sub(parser, ctx)?;
    if expr.is_control_flow() && ctx.at_start_of_statement {
        return Ok(expr);
    }
    while let Some((op_token, rhs)) = parse_op_rhs(parser, ctx, &sub)? {
        expr = combine(Box::new(expr), rhs, op_token);
    }
    Ok(expr)
}

fn parse_logical_or(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    let combine = |lhs, rhs, double_pipe_token| Expr::LogicalOr {
        lhs,
        double_pipe_token,
        rhs,
    };
    parse_binary(parser, ctx, parse_logical_and, combine)
}

fn parse_logical_and(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    let combine = |lhs, rhs, double_ampersand_token| Expr::LogicalAnd {
        lhs,
        double_ampersand_token,
        rhs,
    };
    parse_binary(parser, ctx, parse_comparison, combine)
}

fn parse_comparison(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    let expr = parse_bit_or(parser, ctx)?;
    let expr = if expr.is_control_flow() && ctx.at_start_of_statement {
        expr
    } else if let Some((double_eq_token, rhs)) = parse_op_rhs(parser, ctx, parse_bit_or)? {
        Expr::Equal {
            lhs: Box::new(expr),
            double_eq_token,
            rhs,
        }
    } else if let Some((bang_eq_token, rhs)) = parse_op_rhs(parser, ctx, parse_bit_or)? {
        Expr::NotEqual {
            lhs: Box::new(expr),
            bang_eq_token,
            rhs,
        }
    } else if let Some((less_than_token, rhs)) = parse_op_rhs(parser, ctx, parse_bit_or)? {
        Expr::LessThan {
            lhs: Box::new(expr),
            less_than_token,
            rhs,
        }
    } else if let Some((greater_than_token, rhs)) = parse_op_rhs(parser, ctx, parse_bit_or)? {
        Expr::GreaterThan {
            lhs: Box::new(expr),
            greater_than_token,
            rhs,
        }
    } else if let Some((less_than_eq_token, rhs)) = parse_op_rhs(parser, ctx, parse_bit_or)? {
        Expr::LessThanEq {
            lhs: Box::new(expr),
            less_than_eq_token,
            rhs,
        }
    } else if let Some((greater_than_eq_token, rhs)) = parse_op_rhs(parser, ctx, parse_bit_or)? {
        Expr::GreaterThanEq {
            lhs: Box::new(expr),
            greater_than_eq_token,
            rhs,
        }
    } else {
        expr
    };
    Ok(expr)
}

fn parse_bit_or(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    let combine = |lhs, rhs, pipe_token| Expr::BitOr {
        lhs,
        pipe_token,
        rhs,
    };
    parse_binary(parser, ctx, parse_bit_xor, combine)
}

fn parse_bit_xor(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    let combine = |lhs, rhs, caret_token| Expr::BitXor {
        lhs,
        caret_token,
        rhs,
    };
    parse_binary(parser, ctx, parse_bit_and, combine)
}

fn parse_bit_and(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    let combine = |lhs, rhs, ampersand_token| Expr::BitAnd {
        lhs,
        ampersand_token,
        rhs,
    };
    parse_binary(parser, ctx, parse_shift, combine)
}

fn parse_shift(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    let mut expr = parse_add(parser, ctx)?;
    if expr.is_control_flow() && ctx.at_start_of_statement {
        return Ok(expr);
    }
    loop {
        expr = if let Some((shl_token, rhs)) = parse_op_rhs(parser, ctx, parse_add)? {
            Expr::Shl {
                lhs: Box::new(expr),
                shl_token,
                rhs,
            }
        } else if let Some((shr_token, rhs)) = parse_op_rhs(parser, ctx, parse_add)? {
            Expr::Shr {
                lhs: Box::new(expr),
                shr_token,
                rhs,
            }
        } else {
            return Ok(expr);
        };
    }
}

fn parse_add(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    let mut expr = parse_mul(parser, ctx)?;
    if expr.is_control_flow() && ctx.at_start_of_statement {
        return Ok(expr);
    }
    loop {
        expr = if let Some((add_token, rhs)) = parse_op_rhs(parser, ctx, parse_mul)? {
            Expr::Add {
                lhs: Box::new(expr),
                add_token,
                rhs,
            }
        } else if let Some((sub_token, rhs)) = parse_op_rhs(parser, ctx, parse_mul)? {
            Expr::Sub {
                lhs: Box::new(expr),
                sub_token,
                rhs,
            }
        } else {
            return Ok(expr);
        };
    }
}

fn parse_mul(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    let mut expr = parse_unary_op(parser, ctx)?;
    if expr.is_control_flow() && ctx.at_start_of_statement {
        return Ok(expr);
    }
    loop {
        expr = if let Some((double_star_token, rhs)) = parse_op_rhs(parser, ctx, parse_unary_op)? {
            Expr::Pow {
                lhs: Box::new(expr),
                double_star_token,
                rhs,
            }
        } else if let Some((star_token, rhs)) = parse_op_rhs(parser, ctx, parse_unary_op)? {
            Expr::Mul {
                lhs: Box::new(expr),
                star_token,
                rhs,
            }
        } else if let Some((forward_slash_token, rhs)) = parse_op_rhs(parser, ctx, parse_unary_op)?
        {
            Expr::Div {
                lhs: Box::new(expr),
                forward_slash_token,
                rhs,
            }
        } else if let Some((percent_token, rhs)) = parse_op_rhs(parser, ctx, parse_unary_op)? {
            Expr::Modulo {
                lhs: Box::new(expr),
                percent_token,
                rhs,
            }
        } else {
            return Ok(expr);
        };
    }
}

fn parse_unary_op(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    if let Some((ampersand_token, mut_token, expr)) = parse_referencing(parser, ctx)? {
        return Ok(Expr::Ref {
            ampersand_token,
            mut_token,
            expr,
        });
    }
    if let Some((star_token, expr)) = parse_op_rhs(parser, ctx, parse_unary_op)? {
        return Ok(Expr::Deref { star_token, expr });
    }
    if let Some((bang_token, expr)) = parse_op_rhs(parser, ctx, parse_unary_op)? {
        return Ok(Expr::Not { bang_token, expr });
    }
    return parse_projection(parser, ctx);

    #[allow(clippy::type_complexity)] // Used just here for getting the three parsed elements.
    fn parse_referencing(
        parser: &mut Parser,
        ctx: ParseExprCtx,
    ) -> ParseResult<Option<(AmpersandToken, Option<MutToken>, Box<Expr>)>> {
        if let Some(ampersand_token) = parser.take() {
            let mut_token = parser.take::<MutToken>();
            let expr = Box::new(parse_unary_op(parser, ctx.not_statement())?);
            return Ok(Some((ampersand_token, mut_token, expr)));
        }
        Ok(None)
    }
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

            // Try parsing a field access or a method call.
            if let Some(path_seg) = parser.guarded_parse::<Ident, PathExprSegment>()? {
                if !ctx.parsing_conditional {
                    if let Some(contract_args) = Braces::try_parse(parser)? {
                        expr = Expr::MethodCall {
                            target,
                            dot_token,
                            path_seg,
                            contract_args_opt: Some(contract_args),
                            args: Parens::parse(parser)?,
                        };
                        continue;
                    }
                }
                if let Some(args) = Parens::try_parse(parser)? {
                    expr = Expr::MethodCall {
                        target,
                        dot_token,
                        path_seg,
                        contract_args_opt: None,
                        args,
                    };
                    continue;
                }

                // No arguments, so this is a field projection.
                ensure_field_projection_no_generics(parser, &path_seg.generics_opt);
                expr = Expr::FieldProjection {
                    target,
                    dot_token,
                    name: path_seg.name,
                };
                continue;
            }

            // Try parsing a tuple field projection.
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
                    is_generated_b256: _,
                } = lit_int;
                if ty_opt.is_some() {
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

            // Nothing expected followed. Now we have parsed `expr .`.
            // Try to recover as an unknown sort of expression.
            let err = parser.emit_error(ParseErrorKind::ExpectedFieldName);
            return Ok(Expr::Error([target.span(), dot_token.span()].into(), err));
        }
        return Ok(expr);
    }
}

/// Ensure we don't have `foo.bar::<...>` where `bar` isn't a method call.
fn ensure_field_projection_no_generics(
    parser: &mut Parser,
    generic_args: &Option<(DoubleColonToken, GenericArgs)>,
) {
    if let Some((dct, generic_args)) = generic_args {
        let span = Span::join(dct.span(), &generic_args.span());
        parser.emit_error_with_span(ParseErrorKind::FieldProjectionWithGenericArgs, span);
    }
}

fn parse_func_app(parser: &mut Parser, ctx: ParseExprCtx) -> ParseResult<Expr> {
    let mut expr = parse_atom(parser, ctx)?;
    if expr.is_control_flow() && ctx.at_start_of_statement {
        return Ok(expr);
    }
    while let Some(args) = Parens::try_parse(parser)? {
        let func = Box::new(expr);
        expr = Expr::FuncApp { func, args };
    }
    Ok(expr)
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

    let lit_bool = |span, kind| Ok(Expr::Literal(Literal::Bool(LitBool { span, kind })));
    if let Some(ident) = parser.take::<TrueToken>() {
        return lit_bool(ident.span(), LitBoolType::True);
    }
    if let Some(ident) = parser.take::<FalseToken>() {
        return lit_bool(ident.span(), LitBoolType::False);
    }
    if let Some(asm_block) = parser.guarded_parse::<AsmToken, _>()? {
        return Ok(Expr::Asm(asm_block));
    }
    if let Some(break_token) = parser.take() {
        return Ok(Expr::Break { break_token });
    }
    if let Some(continue_token) = parser.take() {
        return Ok(Expr::Continue { continue_token });
    }
    if let Some(abi_token) = parser.take() {
        let args = parser.parse()?;
        return Ok(Expr::AbiCast { abi_token, args });
    }
    if let Some(return_token) = parser.take() {
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
    if let Some(panic_token) = parser.take() {
        if parser.is_empty()
            || parser.peek::<CommaToken>().is_some()
            || parser.peek::<SemicolonToken>().is_some()
        {
            return Ok(Expr::Panic {
                panic_token,
                expr_opt: None,
            });
        }
        let expr = parser.parse()?;
        return Ok(Expr::Panic {
            panic_token,
            expr_opt: Some(expr),
        });
    }
    if let Some(if_expr) = parser.guarded_parse::<IfToken, _>()? {
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
    if let Some(for_token) = parser.take() {
        let value_pattern = parser.parse()?;
        let in_token = parser.parse()?;
        let iterator = Box::new(parse_condition(parser)?);
        let block = parser.parse()?;
        return Ok(Expr::For {
            for_token,
            value_pattern,
            in_token,
            iterator,
            block,
        });
    }
    if parser.peek::<OpenAngleBracketToken>().is_some()
        || parser.peek::<DoubleColonToken>().is_some()
        || parser.peek::<Ident>().is_some()
    {
        let path: PathExpr = parser.parse()?;
        if path.incomplete_suffix {
            // We tried parsing it as a path but we didn't succeed so we try to recover this
            // as an unknown sort of expression. This happens, for instance, when the user
            // types `foo::`
            return Ok(Expr::Error(
                [path.span()].into(),
                parser.emit_error(ParseErrorKind::ExpectedPathType),
            ));
        }
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
        mut parser: Parser<'a, '_>,
    ) -> ParseResult<(ExprArrayDescriptor, ParserConsumed<'a>)> {
        if let Some(consumed) = parser.check_empty() {
            let punctuated = Punctuated::empty();
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
            let punctuated = Punctuated::single(value);
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
            return Ok(MatchBranchKind::Block {
                block,
                comma_token_opt: parser.take(),
            });
        }
        let expr = parser.parse()?;
        let comma_token = parser.parse()?;
        Ok(MatchBranchKind::Expr { expr, comma_token })
    }
}
