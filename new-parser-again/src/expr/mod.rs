use crate::priv_prelude::*;

pub mod asm;
pub mod op_code;

#[derive(Clone, Debug)]
pub enum Expr {
    Path(PathExpr),
    Literal(Literal),
    //Struct(ExprStruct),
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
        args: Parens<Punctuated<Expr, CommaToken>>,
    },
    FieldProjection {
        target: Box<Expr>,
        dot_token: DotToken,
        name: Ident,
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

#[derive(Clone, Debug)]
pub struct IfExpr {
    pub if_token: IfToken,
    pub condition: Box<Expr>,
    pub then_block: Braces<CodeBlockContents>,
    pub else_opt: Option<(ElseToken, ControlFlow<Braces<CodeBlockContents>, Box<IfExpr>>)>,
}

impl Parse for IfExpr {
    fn parse(parser: &mut Parser) -> ParseResult<IfExpr> {
        println!("parsing if expr: {:#?}", parser.debug_tokens());
        let if_token = parser.parse()?;
        println!("got if token");
        let condition = parser.parse()?;
        println!("got condition");
        let then_block = parser.parse()?;
        println!("got then block");
        let else_opt = match parser.take() {
            Some(else_token) => {
                let else_body = match parser.peek::<IfToken>() {
                    Some(..) => {
                        let if_expr = parser.parse()?;
                        ControlFlow::Continue(Box::new(if_expr))
                    },
                    None => {
                        let else_block = parser.parse()?;
                        ControlFlow::Break(else_block)
                    },
                };
                Some((else_token, else_body))
            },
            None => None,
        };
        Ok(IfExpr { if_token, condition, then_block, else_opt })
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

#[derive(Debug, Clone)]
pub enum MatchBranchKind {
    Block(Braces<CodeBlockContents>),
    Expr {
        expr: Expr,
        comma_token: CommaToken,
    },
}

impl Parse for Expr {
    fn parse(parser: &mut Parser) -> ParseResult<Expr> {
        parse_reassignment(parser)
    }
}

#[derive(Clone, Debug)]
pub struct CodeBlockContents {
    pub statements: Vec<Statement>,
    pub final_expr_opt: Option<Box<Expr>>,
}

impl ParseToEnd for CodeBlockContents {
    fn parse_to_end<'a>(mut parser: Parser<'a>) -> ParseResult<(CodeBlockContents, ParserConsumed<'a>)> {
        let mut statements = Vec::new();
        let (final_expr_opt, consumed) = loop {
            if let Some(consumed) = parser.check_empty() {
                break (None, consumed);
            }
            if {
                parser.peek::<UseToken>().is_some() ||
                parser.peek::<StructToken>().is_some() ||
                parser.peek::<EnumToken>().is_some() ||
                parser.peek::<FnToken>().is_some() ||
                parser.peek::<PubToken>().is_some() ||
                parser.peek::<ImpureToken>().is_some() ||
                parser.peek::<TraitToken>().is_some() ||
                parser.peek::<ImplToken>().is_some() ||
                parser.peek::<AbiToken>().is_some() ||
                parser.peek::<ConstToken>().is_some() ||
                parser.peek::<StorageToken>().is_some()
            } {
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
                    },
                    None => None,
                };
                let eq_token = parser.parse()?;
                let expr = parser.parse()?;
                let semicolon_token = parser.parse()?;
                let statement_let = StatementLet { let_token, pattern, ty_opt, eq_token, expr, semicolon_token };
                let statement = Statement::Let(statement_let);
                statements.push(statement);
                continue;
            }
            let expr = parser.parse::<Expr>()?;
            if let Some(semicolon_token) = parser.take() {
                let statement = Statement::Expr { expr, semicolon_token };
                statements.push(statement);
                continue;
            }
            if let Some(consumed) = parser.check_empty() {
                break (Some(Box::new(expr)), consumed);
            }

            return Err(parser.emit_error("unexpected tokens in statement"));
        };
        let code_block_contents = CodeBlockContents { statements, final_expr_opt };
        Ok((code_block_contents, consumed))
    }
}

fn parse_reassignment(parser: &mut Parser) -> ParseResult<Expr> {
    let expr = parse_logical_or(parser)?;
    if let Some(eq_token) = parser.peek() {
        let assignable = match expr.try_into_assignable() {
            Ok(assignable) => assignable,
            Err(_expr) => {
                // TODO: use expr span
                return Err(parser.emit_error("this expression cannot be assigned to"));
            },
        };
        let expr = Box::new(parse_reassignment(parser)?);
        return Ok(Expr::Reassignment { assignable, eq_token, expr });
    }
    Ok(expr)
}

fn parse_logical_or(parser: &mut Parser) -> ParseResult<Expr> {
    let mut expr = parse_logical_and(parser)?;
    loop {
        if let Some(double_pipe_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_logical_and(parser)?);
            expr = Expr::LogicalOr { lhs, double_pipe_token, rhs };
            continue;
        }
        return Ok(expr);
    }
}

fn parse_logical_and(parser: &mut Parser) -> ParseResult<Expr> {
    let mut expr = parse_comparison(parser)?;
    loop {
        if let Some(double_ampersand_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_comparison(parser)?);
            expr = Expr::LogicalAnd { lhs, double_ampersand_token, rhs };
            continue;
        }
        return Ok(expr);
    }
}

fn parse_comparison(parser: &mut Parser) -> ParseResult<Expr> {
    println!("parsing comparison: {:#?}", parser.debug_tokens());
    let mut expr = parse_bit_or(parser)?;
    loop {
        if let Some(double_eq_token) = parser.take() {
            println!("got double-eq token, parser now at: {:#?}", parser.debug_tokens());
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_bit_or(parser)?);
            println!("got rhs");
            expr = Expr::Equal { lhs, double_eq_token, rhs };
            continue;
        }
        if let Some(bang_eq_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_bit_or(parser)?);
            expr = Expr::NotEqual { lhs, bang_eq_token, rhs };
            continue;
        }
        if let Some(less_than_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_bit_or(parser)?);
            expr = Expr::LessThan { lhs, less_than_token, rhs };
            continue;
        }
        if let Some(greater_than_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_bit_or(parser)?);
            expr = Expr::GreaterThan { lhs, greater_than_token, rhs };
            continue;
        }
        if let Some(less_than_eq_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_bit_or(parser)?);
            expr = Expr::LessThanEq { lhs, less_than_eq_token, rhs };
            continue;
        }
        if let Some(greater_than_eq_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_bit_or(parser)?);
            expr = Expr::GreaterThanEq { lhs, greater_than_eq_token, rhs };
            continue;
        }
        return Ok(expr);
    }
}

fn parse_bit_or(parser: &mut Parser) -> ParseResult<Expr> {
    let mut expr = parse_bit_xor(parser)?;
    loop {
        if let Some(pipe_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_bit_xor(parser)?);
            expr = Expr::BitOr { lhs, pipe_token, rhs };
            continue;
        }
        return Ok(expr);
    }
}

fn parse_bit_xor(parser: &mut Parser) -> ParseResult<Expr> {
    let mut expr = parse_bit_and(parser)?;
    loop {
        if let Some(caret_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_bit_and(parser)?);
            expr = Expr::BitXor { lhs, caret_token, rhs };
            continue;
        }
        return Ok(expr);
    }
}

fn parse_bit_and(parser: &mut Parser) -> ParseResult<Expr> {
    let mut expr = parse_shift(parser)?;
    loop {
        if let Some(ampersand_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_shift(parser)?);
            expr = Expr::BitAnd { lhs, ampersand_token, rhs };
            continue;
        }
        return Ok(expr);
    }
}

fn parse_shift(parser: &mut Parser) -> ParseResult<Expr> {
    let mut expr = parse_add(parser)?;
    loop {
        if let Some(shl_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_add(parser)?);
            expr = Expr::Shl { lhs, shl_token, rhs };
            continue;
        }
        if let Some(shr_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_add(parser)?);
            expr = Expr::Shr { lhs, shr_token, rhs };
            continue;
        }
        return Ok(expr);
    }
}

fn parse_add(parser: &mut Parser) -> ParseResult<Expr> {
    let mut expr = parse_mul(parser)?;
    loop {
        if let Some(add_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_mul(parser)?);
            expr = Expr::Add { lhs, add_token, rhs };
            continue;
        }
        if let Some(sub_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_mul(parser)?);
            expr = Expr::Sub { lhs, sub_token, rhs };
            continue;
        }
        return Ok(expr);
    }
}

fn parse_mul(parser: &mut Parser) -> ParseResult<Expr> {
    let mut expr = parse_unary_op(parser)?;
    loop {
        if let Some(star_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_unary_op(parser)?);
            expr = Expr::Mul { lhs, star_token, rhs };
            continue;
        }
        if let Some(forward_slash_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_unary_op(parser)?);
            expr = Expr::Div { lhs, forward_slash_token, rhs };
            continue;
        }
        if let Some(percent_token) = parser.take() {
            let lhs = Box::new(expr);
            let rhs = Box::new(parse_unary_op(parser)?);
            expr = Expr::Modulo { lhs, percent_token, rhs };
            continue;
        }
        return Ok(expr);
    }
}

fn parse_unary_op(parser: &mut Parser) -> ParseResult<Expr> {
    if let Some(bang_token) = parser.take() {
        let expr = Box::new(parse_unary_op(parser)?);
        return Ok(Expr::Not { bang_token, expr });
    }
    parse_projection(parser)
}

fn parse_projection(parser: &mut Parser) -> ParseResult<Expr> {
    let mut expr = parse_func_app(parser)?;
    loop {
        if let Some(arg) = SquareBrackets::try_parse_all_inner(
            parser,
            |parser| parser.emit_error("unexpected tokens after array index"),
        )? {
            let target = Box::new(expr);
            expr = Expr::Index { target, arg };
            continue;
        }
        if let Some(dot_token) = parser.take() {
            let target = Box::new(expr);
            let name = parser.parse()?;
            if let Some(args) = Parens::try_parse(parser)? {
                expr = Expr::MethodCall { target, dot_token, name, args };
                continue;
            }
            expr = Expr::FieldProjection { target, dot_token, name };
            continue;
        }
        return Ok(expr);
    }
}

fn parse_func_app(parser: &mut Parser) -> ParseResult<Expr> {
    let mut expr = parse_atom(parser)?;
    loop {
        if let Some(args) = Parens::try_parse(parser)? {
            let func = Box::new(expr);
            expr = Expr::FuncApp { func, args };
            continue;
        }
        return Ok(expr);
    }
}

fn parse_atom(parser: &mut Parser) -> ParseResult<Expr> {
    println!("parsing atom: {:#?}", parser.debug_tokens());
    if let Some(code_block_inner) = Braces::try_parse(parser)? {
        return Ok(Expr::Block(code_block_inner));
    }
    if let Some(array_inner) = SquareBrackets::try_parse(parser)? {
        return Ok(Expr::Array(array_inner));
    }
    if let Some(mut parser) = parser.enter_delimited(Delimiter::Parenthesis) {
        if let Some(consumed) = parser.check_empty() {
            return Ok(Expr::Tuple(Parens::new(ExprTupleDescriptor::Nil, consumed)));
        }
        let head = parser.parse()?;
        if let Some(comma_token) = parser.take() {
            let (tail, consumed) = parser.parse_to_end()?;
            let tuple = ExprTupleDescriptor::Cons { head, comma_token, tail };
            return Ok(Expr::Tuple(Parens::new(tuple, consumed)));
        }
        if let Some(consumed) = parser.check_empty() {
            return Ok(Expr::Parens(Parens::new(head, consumed)));
        }
        return Err(parser.emit_error("expected a comma (if this is meant to be a tuple), or a closing parenthesis."));
    }
    if parser.peek::<AsmToken>().is_some() {
        let asm_block = parser.parse()?;
        return Ok(Expr::Asm(asm_block));
    }
    if let Some(return_token) = parser.take() {
        // TODO: how to handle this properly?
        if {
            parser.is_empty() ||
            parser.peek::<CommaToken>().is_some() ||
            parser.peek::<SemicolonToken>().is_some()
        } {
            return Ok(Expr::Return { return_token, expr_opt: None });
        }
        let expr = parser.parse()?;
        return Ok(Expr::Return { return_token, expr_opt: Some(expr) });
    }
    if parser.peek::<IfToken>().is_some() {
        let if_expr = parser.parse()?;
        return Ok(Expr::If(if_expr));
    }
    if let Some(match_token) = parser.take() {
        let condition = parser.parse()?;
        let branches = parser.parse()?;
        return Ok(Expr::Match { match_token, condition, branches });
    }
    if {
        parser.peek::<LessThanToken>().is_some() ||
        parser.peek::<DoubleColonToken>().is_some() ||
        parser.peek::<Ident>().is_some()
    } {
        println!("parsing atom, looks like an ident: {:#?}", parser.debug_tokens());
        let path = parser.parse()?;
        let expr = match Braces::try_parse(parser)? {
            Some(fields) => Expr::Struct { path, fields },
            None => Expr::Path(path),
        };
        return Ok(expr);
    }
    if let Some(literal) = parser.take() {
        return Ok(Expr::Literal(literal));
    }
    Err(parser.emit_error("expected an expression"))
}

#[derive(Clone, Debug)]
pub struct ExprStructField  {
    pub field_name: Ident,
    pub expr_opt: Option<(ColonToken, Box<Expr>)>,
}

impl Parse for ExprStructField {
    fn parse(parser: &mut Parser) -> ParseResult<ExprStructField> {
        let field_name = parser.parse()?;
        let expr_opt = match parser.take() {
            Some(colon_token) => {
                let expr = parser.parse()?;
                Some((colon_token, expr))
            },
            None => None,
        };
        Ok(ExprStructField { field_name, expr_opt })
    }
}

impl ParseToEnd for ExprArrayDescriptor {
    fn parse_to_end<'a>(mut parser: Parser<'a>) -> ParseResult<(ExprArrayDescriptor, ParserConsumed<'a>)> {
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
                    return Err(parser.emit_error("unexpected tokens after array length"));
                },
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
            punctuated.value_separator_pairs.insert(0, (value, comma_token));
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
        Err(parser.emit_error("unexpected tokens parsing array expression. Expected a comma, semicolon or closing square bracket."))
    }
}

impl Parse for MatchBranch {
    fn parse(parser: &mut Parser) -> ParseResult<MatchBranch> {
        let pattern = parser.parse()?;
        let fat_right_arrow_token = parser.parse()?;
        let kind = parser.parse()?;
        Ok(MatchBranch { pattern, fat_right_arrow_token, kind })
    }
}

impl Parse for MatchBranchKind {
    fn parse(parser: &mut Parser) -> ParseResult<MatchBranchKind> {
        if let Some(block) = Braces::try_parse(parser)? {
            return Ok(MatchBranchKind::Block(block));
        }
        let expr = parser.parse()?;
        let comma_token = parser.parse()?;
        Ok(MatchBranchKind::Expr { expr, comma_token })
    }
}

impl Expr {
    pub fn try_into_assignable(self) -> Result<Assignable, Expr> {
        match self {
            Expr::Path(path_expr) => {
                match path_expr.try_into_ident() {
                    Ok(name) => Ok(Assignable::Var(name)),
                    Err(path_expr) => Err(Expr::Path(path_expr)),
                }
            },
            Expr::Index { target, arg } => {
                match target.try_into_assignable() {
                    Ok(target) => {
                        Ok(Assignable::Index { target: Box::new(target), arg })
                    },
                    Err(target) => {
                        Err(Expr::Index { target: Box::new(target), arg })
                    },
                }
            },
            Expr::FieldProjection { target, dot_token, name } => {
                match target.try_into_assignable() {
                    Ok(target) => {
                        Ok(Assignable::FieldProjection { target: Box::new(target), dot_token, name })
                    },
                    Err(target) => {
                        Err(Expr::FieldProjection { target: Box::new(target), dot_token, name })
                    },
                }
            },
            expr => Err(expr),
        }
    }
}

