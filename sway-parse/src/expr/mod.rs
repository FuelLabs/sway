use crate::priv_prelude::*;

pub mod asm;
pub mod op_code;

#[derive(Clone, Debug)]
pub enum Expr {
    Path(PathExpr),
    Literal(Literal),
    AbiCast {
        abi_token: AbiToken,
        args: Parens<AbiCastArgs>,
    },
    Struct {
        path: PathExpr,
        fields: Braces<Punctuated<ExprStructField, CommaToken>>,
    },
    Tuple(Parens<ExprTupleDescriptor>),
    Parens(Parens<Box<Expr>>),
    Block(Braces<CodeBlockContents>),
    Array(SquareBrackets<ExprArrayDescriptor>),
    Asm(AsmBlock),
    Return {
        return_token: ReturnToken,
        expr_opt: Option<Box<Expr>>,
    },
    If(IfExpr),
    Match {
        match_token: MatchToken,
        condition: Box<Expr>,
        branches: Braces<Vec<MatchBranch>>,
    },
    While {
        while_token: WhileToken,
        condition: Box<Expr>,
        block: Braces<CodeBlockContents>,
    },
    FuncApp {
        func: Box<Expr>,
        args: Parens<Punctuated<Expr, CommaToken>>,
    },
    Index {
        target: Box<Expr>,
        arg: SquareBrackets<Box<Expr>>,
    },
    MethodCall {
        target: Box<Expr>,
        dot_token: DotToken,
        name: Ident,
        contract_args_opt: Option<Braces<Punctuated<ExprStructField, CommaToken>>>,
        args: Parens<Punctuated<Expr, CommaToken>>,
    },
    FieldProjection {
        target: Box<Expr>,
        dot_token: DotToken,
        name: Ident,
    },
    TupleFieldProjection {
        target: Box<Expr>,
        dot_token: DotToken,
        field: BigUint,
        field_span: Span,
    },
    Ref {
        ref_token: RefToken,
        expr: Box<Expr>,
    },
    Deref {
        deref_token: DerefToken,
        expr: Box<Expr>,
    },
    Not {
        bang_token: BangToken,
        expr: Box<Expr>,
    },
    Mul {
        lhs: Box<Expr>,
        star_token: StarToken,
        rhs: Box<Expr>,
    },
    Div {
        lhs: Box<Expr>,
        forward_slash_token: ForwardSlashToken,
        rhs: Box<Expr>,
    },
    Modulo {
        lhs: Box<Expr>,
        percent_token: PercentToken,
        rhs: Box<Expr>,
    },
    Add {
        lhs: Box<Expr>,
        add_token: AddToken,
        rhs: Box<Expr>,
    },
    Sub {
        lhs: Box<Expr>,
        sub_token: SubToken,
        rhs: Box<Expr>,
    },
    Shl {
        lhs: Box<Expr>,
        shl_token: ShlToken,
        rhs: Box<Expr>,
    },
    Shr {
        lhs: Box<Expr>,
        shr_token: ShrToken,
        rhs: Box<Expr>,
    },
    BitAnd {
        lhs: Box<Expr>,
        ampersand_token: AmpersandToken,
        rhs: Box<Expr>,
    },
    BitXor {
        lhs: Box<Expr>,
        caret_token: CaretToken,
        rhs: Box<Expr>,
    },
    BitOr {
        lhs: Box<Expr>,
        pipe_token: PipeToken,
        rhs: Box<Expr>,
    },
    Equal {
        lhs: Box<Expr>,
        double_eq_token: DoubleEqToken,
        rhs: Box<Expr>,
    },
    NotEqual {
        lhs: Box<Expr>,
        bang_eq_token: BangEqToken,
        rhs: Box<Expr>,
    },
    LessThan {
        lhs: Box<Expr>,
        less_than_token: LessThanToken,
        rhs: Box<Expr>,
    },
    GreaterThan {
        lhs: Box<Expr>,
        greater_than_token: GreaterThanToken,
        rhs: Box<Expr>,
    },
    LessThanEq {
        lhs: Box<Expr>,
        less_than_eq_token: LessThanEqToken,
        rhs: Box<Expr>,
    },
    GreaterThanEq {
        lhs: Box<Expr>,
        greater_than_eq_token: GreaterThanEqToken,
        rhs: Box<Expr>,
    },
    LogicalAnd {
        lhs: Box<Expr>,
        double_ampersand_token: DoubleAmpersandToken,
        rhs: Box<Expr>,
    },
    LogicalOr {
        lhs: Box<Expr>,
        double_pipe_token: DoublePipeToken,
        rhs: Box<Expr>,
    },
    Reassignment {
        assignable: Assignable,
        eq_token: EqToken,
        expr: Box<Expr>,
    },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Path(path_expr) => path_expr.span(),
            Expr::Literal(literal) => literal.span(),
            Expr::AbiCast { abi_token, args } => Span::join(abi_token.span(), args.span()),
            Expr::Struct { path, fields } => Span::join(path.span(), fields.span()),
            Expr::Tuple(tuple_expr) => tuple_expr.span(),
            Expr::Parens(parens) => parens.span(),
            Expr::Block(block_expr) => block_expr.span(),
            Expr::Array(array_expr) => array_expr.span(),
            Expr::Asm(asm_block) => asm_block.span(),
            Expr::Return {
                return_token,
                expr_opt,
            } => {
                let start = return_token.span();
                let end = match expr_opt {
                    Some(expr) => expr.span(),
                    None => return_token.span(),
                };
                Span::join(start, end)
            }
            Expr::If(if_expr) => if_expr.span(),
            Expr::Match {
                match_token,
                branches,
                ..
            } => Span::join(match_token.span(), branches.span()),
            Expr::While {
                while_token, block, ..
            } => Span::join(while_token.span(), block.span()),
            Expr::FuncApp { func, args } => Span::join(func.span(), args.span()),
            Expr::Index { target, arg } => Span::join(target.span(), arg.span()),
            Expr::MethodCall { target, args, .. } => Span::join(target.span(), args.span()),
            Expr::FieldProjection { target, name, .. } => {
                Span::join(target.span(), name.span().clone())
            }
            Expr::TupleFieldProjection {
                target, field_span, ..
            } => Span::join(target.span(), field_span.clone()),
            Expr::Ref { ref_token, expr } => Span::join(ref_token.span(), expr.span()),
            Expr::Deref { deref_token, expr } => Span::join(deref_token.span(), expr.span()),
            Expr::Not { bang_token, expr } => Span::join(bang_token.span(), expr.span()),
            Expr::Mul { lhs, rhs, .. } => Span::join(lhs.span(), rhs.span()),
            Expr::Div { lhs, rhs, .. } => Span::join(lhs.span(), rhs.span()),
            Expr::Modulo { lhs, rhs, .. } => Span::join(lhs.span(), rhs.span()),
            Expr::Add { lhs, rhs, .. } => Span::join(lhs.span(), rhs.span()),
            Expr::Sub { lhs, rhs, .. } => Span::join(lhs.span(), rhs.span()),
            Expr::Shl { lhs, rhs, .. } => Span::join(lhs.span(), rhs.span()),
            Expr::Shr { lhs, rhs, .. } => Span::join(lhs.span(), rhs.span()),
            Expr::BitAnd { lhs, rhs, .. } => Span::join(lhs.span(), rhs.span()),
            Expr::BitXor { lhs, rhs, .. } => Span::join(lhs.span(), rhs.span()),
            Expr::BitOr { lhs, rhs, .. } => Span::join(lhs.span(), rhs.span()),
            Expr::Equal { lhs, rhs, .. } => Span::join(lhs.span(), rhs.span()),
            Expr::NotEqual { lhs, rhs, .. } => Span::join(lhs.span(), rhs.span()),
            Expr::LessThan { lhs, rhs, .. } => Span::join(lhs.span(), rhs.span()),
            Expr::GreaterThan { lhs, rhs, .. } => Span::join(lhs.span(), rhs.span()),
            Expr::LessThanEq { lhs, rhs, .. } => Span::join(lhs.span(), rhs.span()),
            Expr::GreaterThanEq { lhs, rhs, .. } => Span::join(lhs.span(), rhs.span()),
            Expr::LogicalAnd { lhs, rhs, .. } => Span::join(lhs.span(), rhs.span()),
            Expr::LogicalOr { lhs, rhs, .. } => Span::join(lhs.span(), rhs.span()),
            Expr::Reassignment {
                assignable, expr, ..
            } => Span::join(assignable.span(), expr.span()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AbiCastArgs {
    pub name: PathType,
    pub comma_token: CommaToken,
    pub address: Box<Expr>,
}

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

#[allow(clippy::type_complexity)]
#[derive(Clone, Debug)]
pub struct IfExpr {
    pub if_token: IfToken,
    pub condition: IfCondition,
    pub then_block: Braces<CodeBlockContents>,
    pub else_opt: Option<(
        ElseToken,
        ControlFlow<Braces<CodeBlockContents>, Box<IfExpr>>,
    )>,
}

#[derive(Clone, Debug)]
pub enum IfCondition {
    Expr(Box<Expr>),
    Let {
        let_token: LetToken,
        lhs: Box<Pattern>,
        eq_token: EqToken,
        rhs: Box<Expr>,
    },
}

impl IfExpr {
    pub fn span(&self) -> Span {
        let start = self.if_token.span();
        let end = match &self.else_opt {
            Some((_else_token, tail)) => match tail {
                ControlFlow::Break(block) => block.span(),
                ControlFlow::Continue(if_expr) => if_expr.span(),
            },
            None => self.then_block.span(),
        };
        Span::join(start, end)
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

#[derive(Clone, Debug)]
pub enum ExprTupleDescriptor {
    Nil,
    Cons {
        head: Box<Expr>,
        comma_token: CommaToken,
        tail: Punctuated<Expr, CommaToken>,
    },
}

#[derive(Clone, Debug)]
pub enum ExprArrayDescriptor {
    Sequence(Punctuated<Expr, CommaToken>),
    Repeat {
        value: Box<Expr>,
        semicolon_token: SemicolonToken,
        length: Box<Expr>,
    },
}

#[derive(Clone, Debug)]
pub struct MatchBranch {
    pub pattern: Pattern,
    pub fat_right_arrow_token: FatRightArrowToken,
    pub kind: MatchBranchKind,
}

impl MatchBranch {
    pub fn span(&self) -> Span {
        Span::join(self.pattern.span(), self.kind.span())
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum MatchBranchKind {
    Block {
        block: Braces<CodeBlockContents>,
        comma_token_opt: Option<CommaToken>,
    },
    Expr {
        expr: Expr,
        comma_token: CommaToken,
    },
}

impl MatchBranchKind {
    pub fn span(&self) -> Span {
        match self {
            MatchBranchKind::Block {
                block,
                comma_token_opt,
            } => match comma_token_opt {
                Some(comma_token) => Span::join(block.span(), comma_token.span()),
                None => block.span(),
            },
            MatchBranchKind::Expr { expr, comma_token } => {
                Span::join(expr.span(), comma_token.span())
            }
        }
    }
}

impl Parse for Expr {
    fn parse(parser: &mut Parser) -> ParseResult<Expr> {
        parse_reassignment(parser, true)
    }
}

#[derive(Clone, Debug)]
pub struct CodeBlockContents {
    pub statements: Vec<Statement>,
    pub final_expr_opt: Option<Box<Expr>>,
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
                || parser.peek::<ImpureToken>().is_some()
                || parser.peek::<TraitToken>().is_some()
                || parser.peek::<ImplToken>().is_some()
                || parser.peek2::<AbiToken, Ident>().is_some()
                || parser.peek::<ConstToken>().is_some()
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
            let expr = parser.parse::<Expr>()?;
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

fn parse_condition(parser: &mut Parser) -> ParseResult<Expr> {
    parse_reassignment(parser, false)
}

fn parse_reassignment(parser: &mut Parser, allow_struct_exprs: bool) -> ParseResult<Expr> {
    let expr = parse_logical_or(parser, allow_struct_exprs)?;
    if let Some(eq_token) = parser.take() {
        let assignable = match expr.try_into_assignable() {
            Ok(assignable) => assignable,
            Err(expr) => {
                let span = expr.span();
                return Err(
                    parser.emit_error_with_span(ParseErrorKind::UnassignableExpression, span)
                );
            }
        };
        let expr = Box::new(parse_reassignment(parser, allow_struct_exprs)?);
        return Ok(Expr::Reassignment {
            assignable,
            eq_token,
            expr,
        });
    }
    Ok(expr)
}

fn parse_logical_or(parser: &mut Parser, allow_struct_exprs: bool) -> ParseResult<Expr> {
    let mut expr = parse_logical_and(parser, allow_struct_exprs)?;
    loop {
        if let Some(double_pipe_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_logical_and(parser, allow_struct_exprs)?);
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

fn parse_logical_and(parser: &mut Parser, allow_struct_exprs: bool) -> ParseResult<Expr> {
    let mut expr = parse_comparison(parser, allow_struct_exprs)?;
    loop {
        if let Some(double_ampersand_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_comparison(parser, allow_struct_exprs)?);
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

fn parse_comparison(parser: &mut Parser, allow_struct_exprs: bool) -> ParseResult<Expr> {
    let mut expr = parse_bit_or(parser, allow_struct_exprs)?;
    loop {
        if let Some(double_eq_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_bit_or(parser, allow_struct_exprs)?);
            expr = Expr::Equal {
                lhs,
                double_eq_token,
                rhs,
            };
            continue;
        }
        if let Some(bang_eq_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_bit_or(parser, allow_struct_exprs)?);
            expr = Expr::NotEqual {
                lhs,
                bang_eq_token,
                rhs,
            };
            continue;
        }
        if let Some(less_than_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_bit_or(parser, allow_struct_exprs)?);
            expr = Expr::LessThan {
                lhs,
                less_than_token,
                rhs,
            };
            continue;
        }
        if let Some(greater_than_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_bit_or(parser, allow_struct_exprs)?);
            expr = Expr::GreaterThan {
                lhs,
                greater_than_token,
                rhs,
            };
            continue;
        }
        if let Some(less_than_eq_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_bit_or(parser, allow_struct_exprs)?);
            expr = Expr::LessThanEq {
                lhs,
                less_than_eq_token,
                rhs,
            };
            continue;
        }
        if let Some(greater_than_eq_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_bit_or(parser, allow_struct_exprs)?);
            expr = Expr::GreaterThanEq {
                lhs,
                greater_than_eq_token,
                rhs,
            };
            continue;
        }
        return Ok(expr);
    }
}

fn parse_bit_or(parser: &mut Parser, allow_struct_exprs: bool) -> ParseResult<Expr> {
    let mut expr = parse_bit_xor(parser, allow_struct_exprs)?;
    loop {
        if let Some(pipe_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_bit_xor(parser, allow_struct_exprs)?);
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

fn parse_bit_xor(parser: &mut Parser, allow_struct_exprs: bool) -> ParseResult<Expr> {
    let mut expr = parse_bit_and(parser, allow_struct_exprs)?;
    loop {
        if let Some(caret_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_bit_and(parser, allow_struct_exprs)?);
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

fn parse_bit_and(parser: &mut Parser, allow_struct_exprs: bool) -> ParseResult<Expr> {
    let mut expr = parse_shift(parser, allow_struct_exprs)?;
    loop {
        if let Some(ampersand_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_shift(parser, allow_struct_exprs)?);
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

fn parse_shift(parser: &mut Parser, allow_struct_exprs: bool) -> ParseResult<Expr> {
    let mut expr = parse_add(parser, allow_struct_exprs)?;
    loop {
        if let Some(shl_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_add(parser, allow_struct_exprs)?);
            expr = Expr::Shl {
                lhs,
                shl_token,
                rhs,
            };
            continue;
        }
        if let Some(shr_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_add(parser, allow_struct_exprs)?);
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

fn parse_add(parser: &mut Parser, allow_struct_exprs: bool) -> ParseResult<Expr> {
    let mut expr = parse_mul(parser, allow_struct_exprs)?;
    loop {
        if let Some(add_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_mul(parser, allow_struct_exprs)?);
            expr = Expr::Add {
                lhs,
                add_token,
                rhs,
            };
            continue;
        }
        if let Some(sub_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_mul(parser, allow_struct_exprs)?);
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

fn parse_mul(parser: &mut Parser, allow_struct_exprs: bool) -> ParseResult<Expr> {
    let mut expr = parse_unary_op(parser, allow_struct_exprs)?;
    loop {
        if let Some(star_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_unary_op(parser, allow_struct_exprs)?);
            expr = Expr::Mul {
                lhs,
                star_token,
                rhs,
            };
            continue;
        }
        if let Some(forward_slash_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_unary_op(parser, allow_struct_exprs)?);
            expr = Expr::Div {
                lhs,
                forward_slash_token,
                rhs,
            };
            continue;
        }
        if let Some(percent_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_unary_op(parser, allow_struct_exprs)?);
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

fn parse_unary_op(parser: &mut Parser, allow_struct_exprs: bool) -> ParseResult<Expr> {
    if let Some(ref_token) = parser.take() {
        let expr = Box::new(parse_unary_op(parser, allow_struct_exprs)?);
        return Ok(Expr::Ref { ref_token, expr });
    }
    if let Some(deref_token) = parser.take() {
        let expr = Box::new(parse_unary_op(parser, allow_struct_exprs)?);
        return Ok(Expr::Deref { deref_token, expr });
    }
    if let Some(bang_token) = parser.take() {
        let expr = Box::new(parse_unary_op(parser, allow_struct_exprs)?);
        return Ok(Expr::Not { bang_token, expr });
    }
    parse_projection(parser, allow_struct_exprs)
}

fn parse_projection(parser: &mut Parser, allow_struct_exprs: bool) -> ParseResult<Expr> {
    let mut expr = parse_func_app(parser, allow_struct_exprs)?;
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
                if allow_struct_exprs {
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

fn parse_func_app(parser: &mut Parser, allow_struct_exprs: bool) -> ParseResult<Expr> {
    let mut expr = parse_atom(parser, allow_struct_exprs)?;
    loop {
        if let Some(args) = Parens::try_parse(parser)? {
            let func = Box::new(expr);
            expr = Expr::FuncApp { func, args };
            continue;
        }
        return Ok(expr);
    }
}

fn parse_atom(parser: &mut Parser, allow_struct_exprs: bool) -> ParseResult<Expr> {
    if let Some(code_block_inner) = Braces::try_parse(parser)? {
        return Ok(Expr::Block(code_block_inner));
    }
    if let Some(array_inner) = SquareBrackets::try_parse(parser)? {
        return Ok(Expr::Array(array_inner));
    }
    if let Some((mut parser, span)) = parser.enter_delimited(Delimiter::Parenthesis) {
        if let Some(consumed) = parser.check_empty() {
            return Ok(Expr::Tuple(Parens::new(
                ExprTupleDescriptor::Nil,
                span,
                consumed,
            )));
        }
        let head = parser.parse()?;
        if let Some(comma_token) = parser.take() {
            let (tail, consumed) = parser.parse_to_end()?;
            let tuple = ExprTupleDescriptor::Cons {
                head,
                comma_token,
                tail,
            };
            return Ok(Expr::Tuple(Parens::new(tuple, span, consumed)));
        }
        if let Some(consumed) = parser.check_empty() {
            return Ok(Expr::Parens(Parens::new(head, span, consumed)));
        }
        return Err(
            parser.emit_error(ParseErrorKind::ExpectedCommaOrCloseParenInTupleOrParenExpression)
        );
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
            condition,
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
        if allow_struct_exprs {
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

#[derive(Clone, Debug)]
pub struct ExprStructField {
    pub field_name: Ident,
    pub expr_opt: Option<(ColonToken, Box<Expr>)>,
}

impl ExprStructField {
    pub fn span(&self) -> Span {
        match &self.expr_opt {
            None => self.field_name.span().clone(),
            Some((_colon_token, expr)) => Span::join(self.field_name.span().clone(), expr.span()),
        }
    }
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

impl Expr {
    pub fn try_into_assignable(self) -> Result<Assignable, Expr> {
        match self {
            Expr::Path(path_expr) => match path_expr.try_into_ident() {
                Ok(name) => Ok(Assignable::Var(name)),
                Err(path_expr) => Err(Expr::Path(path_expr)),
            },
            Expr::Index { target, arg } => match target.try_into_assignable() {
                Ok(target) => Ok(Assignable::Index {
                    target: Box::new(target),
                    arg,
                }),
                Err(target) => Err(Expr::Index {
                    target: Box::new(target),
                    arg,
                }),
            },
            Expr::FieldProjection {
                target,
                dot_token,
                name,
            } => match target.try_into_assignable() {
                Ok(target) => Ok(Assignable::FieldProjection {
                    target: Box::new(target),
                    dot_token,
                    name,
                }),
                Err(target) => Err(Expr::FieldProjection {
                    target: Box::new(target),
                    dot_token,
                    name,
                }),
            },
            expr => Err(expr),
        }
    }

    pub fn is_control_flow(&self) -> bool {
        matches!(
            self,
            Expr::Block(..)
                | Expr::Asm(..)
                | Expr::If(..)
                | Expr::Match { .. }
                | Expr::While { .. },
        )
    }
}
