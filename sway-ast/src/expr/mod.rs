use crate::priv_prelude::*;

pub mod asm;
pub mod op_code;

#[derive(Clone, Debug)]
pub enum Expr {
    /// A malformed expression.
    ///
    /// Used for parser recovery when we cannot form a more specific node.
    Error(Span),
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
        value: Box<Expr>,
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
        reassignment_op: ReassignmentOp,
        expr: Box<Expr>,
    },
    Break {
        break_token: BreakToken,
    },
    Continue {
        continue_token: ContinueToken,
    },
}

impl Spanned for Expr {
    fn span(&self) -> Span {
        match self {
            Expr::Error(span) => span.clone(),
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
            Expr::FieldProjection { target, name, .. } => Span::join(target.span(), name.span()),
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
            Expr::Break { break_token } => break_token.span(),
            Expr::Continue { continue_token } => continue_token.span(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ReassignmentOp {
    pub variant: ReassignmentOpVariant,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ReassignmentOpVariant {
    Equals,
    AddEquals,
    SubEquals,
    MulEquals,
    DivEquals,
    ShlEquals,
    ShrEquals,
}

impl ReassignmentOpVariant {
    pub fn core_name(&self) -> &'static str {
        match self {
            ReassignmentOpVariant::Equals => "eq",
            ReassignmentOpVariant::AddEquals => "add",
            ReassignmentOpVariant::SubEquals => "subtract",
            ReassignmentOpVariant::MulEquals => "multiply",
            ReassignmentOpVariant::DivEquals => "divide",
            ReassignmentOpVariant::ShlEquals => "lsh",
            ReassignmentOpVariant::ShrEquals => "rsh",
        }
    }
}

#[derive(Clone, Debug)]
pub struct AbiCastArgs {
    pub name: PathType,
    pub comma_token: CommaToken,
    pub address: Box<Expr>,
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

impl Spanned for IfExpr {
    fn span(&self) -> Span {
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

impl Spanned for MatchBranch {
    fn span(&self) -> Span {
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

impl Spanned for MatchBranchKind {
    fn span(&self) -> Span {
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

#[derive(Clone, Debug)]
pub struct CodeBlockContents {
    pub statements: Vec<Statement>,
    pub final_expr_opt: Option<Box<Expr>>,
}

#[derive(Clone, Debug)]
pub struct ExprStructField {
    pub field_name: Ident,
    pub expr_opt: Option<(ColonToken, Box<Expr>)>,
}

impl Spanned for ExprStructField {
    fn span(&self) -> Span {
        match &self.expr_opt {
            None => self.field_name.span(),
            Some((_colon_token, expr)) => Span::join(self.field_name.span(), expr.span()),
        }
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
            Expr::TupleFieldProjection {
                target,
                dot_token,
                field,
                field_span,
            } => match target.try_into_assignable() {
                Ok(target) => Ok(Assignable::TupleFieldProjection {
                    target: Box::new(target),
                    dot_token,
                    field,
                    field_span,
                }),
                Err(target) => Err(Expr::TupleFieldProjection {
                    target: Box::new(target),
                    dot_token,
                    field,
                    field_span,
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
