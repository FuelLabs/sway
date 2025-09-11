use sway_error::handler::ErrorEmitted;

use crate::{assignable::ElementAccess, priv_prelude::*, PathExprSegment};

pub mod asm;
pub mod op_code;

#[derive(Clone, Debug, Serialize)]
pub enum Expr {
    /// A malformed expression.
    ///
    /// Used for parser recovery when we cannot form a more specific node.
    Error(Box<[Span]>, #[serde(skip_serializing)] ErrorEmitted),
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
    Panic {
        panic_token: PanicToken,
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
    For {
        for_token: ForToken,
        in_token: InToken,
        value_pattern: Pattern,
        iterator: Box<Expr>,
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
        path_seg: PathExprSegment,
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
        ampersand_token: AmpersandToken,
        mut_token: Option<MutToken>,
        expr: Box<Expr>,
    },
    Deref {
        star_token: StarToken,
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
    Pow {
        lhs: Box<Expr>,
        double_star_token: DoubleStarToken,
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
            Expr::Error(spans, _) => spans
                .iter()
                .cloned()
                .reduce(|s1: Span, s2: Span| Span::join(s1, &s2))
                .unwrap(),
            Expr::Path(path_expr) => path_expr.span(),
            Expr::Literal(literal) => literal.span(),
            Expr::AbiCast { abi_token, args } => Span::join(abi_token.span(), &args.span()),
            Expr::Struct { path, fields } => Span::join(path.span(), &fields.span()),
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
                Span::join(start, &end)
            }
            Expr::Panic {
                panic_token,
                expr_opt,
            } => {
                let start = panic_token.span();
                let end = match expr_opt {
                    Some(expr) => expr.span(),
                    None => panic_token.span(),
                };
                Span::join(start, &end)
            }
            Expr::If(if_expr) => if_expr.span(),
            Expr::Match {
                match_token,
                branches,
                ..
            } => Span::join(match_token.span(), &branches.span()),
            Expr::While {
                while_token, block, ..
            } => Span::join(while_token.span(), &block.span()),
            Expr::For {
                for_token, block, ..
            } => Span::join(for_token.span(), &block.span()),
            Expr::FuncApp { func, args } => Span::join(func.span(), &args.span()),
            Expr::Index { target, arg } => Span::join(target.span(), &arg.span()),
            Expr::MethodCall { target, args, .. } => Span::join(target.span(), &args.span()),
            Expr::FieldProjection { target, name, .. } => Span::join(target.span(), &name.span()),
            Expr::TupleFieldProjection {
                target, field_span, ..
            } => Span::join(target.span(), field_span),
            Expr::Ref {
                ampersand_token,
                expr,
                ..
            } => Span::join(ampersand_token.span(), &expr.span()),
            Expr::Deref { star_token, expr } => Span::join(star_token.span(), &expr.span()),
            Expr::Not { bang_token, expr } => Span::join(bang_token.span(), &expr.span()),
            Expr::Pow { lhs, rhs, .. } => Span::join(lhs.span(), &rhs.span()),
            Expr::Mul { lhs, rhs, .. } => Span::join(lhs.span(), &rhs.span()),
            Expr::Div { lhs, rhs, .. } => Span::join(lhs.span(), &rhs.span()),
            Expr::Modulo { lhs, rhs, .. } => Span::join(lhs.span(), &rhs.span()),
            Expr::Add { lhs, rhs, .. } => Span::join(lhs.span(), &rhs.span()),
            Expr::Sub { lhs, rhs, .. } => Span::join(lhs.span(), &rhs.span()),
            Expr::Shl { lhs, rhs, .. } => Span::join(lhs.span(), &rhs.span()),
            Expr::Shr { lhs, rhs, .. } => Span::join(lhs.span(), &rhs.span()),
            Expr::BitAnd { lhs, rhs, .. } => Span::join(lhs.span(), &rhs.span()),
            Expr::BitXor { lhs, rhs, .. } => Span::join(lhs.span(), &rhs.span()),
            Expr::BitOr { lhs, rhs, .. } => Span::join(lhs.span(), &rhs.span()),
            Expr::Equal { lhs, rhs, .. } => Span::join(lhs.span(), &rhs.span()),
            Expr::NotEqual { lhs, rhs, .. } => Span::join(lhs.span(), &rhs.span()),
            Expr::LessThan { lhs, rhs, .. } => Span::join(lhs.span(), &rhs.span()),
            Expr::GreaterThan { lhs, rhs, .. } => Span::join(lhs.span(), &rhs.span()),
            Expr::LessThanEq { lhs, rhs, .. } => Span::join(lhs.span(), &rhs.span()),
            Expr::GreaterThanEq { lhs, rhs, .. } => Span::join(lhs.span(), &rhs.span()),
            Expr::LogicalAnd { lhs, rhs, .. } => Span::join(lhs.span(), &rhs.span()),
            Expr::LogicalOr { lhs, rhs, .. } => Span::join(lhs.span(), &rhs.span()),
            Expr::Reassignment {
                assignable, expr, ..
            } => Span::join(assignable.span(), &expr.span()),
            Expr::Break { break_token } => break_token.span(),
            Expr::Continue { continue_token } => continue_token.span(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ReassignmentOp {
    pub variant: ReassignmentOpVariant,
    pub span: Span,
}

#[derive(Clone, Debug, Serialize)]
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
    pub fn std_name(&self) -> &'static str {
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

    pub fn as_str(&self) -> &'static str {
        match self {
            ReassignmentOpVariant::Equals => EqToken::AS_STR,
            ReassignmentOpVariant::AddEquals => AddEqToken::AS_STR,
            ReassignmentOpVariant::SubEquals => SubEqToken::AS_STR,
            ReassignmentOpVariant::MulEquals => StarEqToken::AS_STR,
            ReassignmentOpVariant::DivEquals => DivEqToken::AS_STR,
            ReassignmentOpVariant::ShlEquals => ShlEqToken::AS_STR,
            ReassignmentOpVariant::ShrEquals => ShrEqToken::AS_STR,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct AbiCastArgs {
    pub name: PathType,
    pub comma_token: CommaToken,
    pub address: Box<Expr>,
}

#[allow(clippy::type_complexity)]
#[derive(Clone, Debug, Serialize)]
pub struct IfExpr {
    pub if_token: IfToken,
    pub condition: IfCondition,
    pub then_block: Braces<CodeBlockContents>,
    pub else_opt: Option<(
        ElseToken,
        LoopControlFlow<Braces<CodeBlockContents>, Box<IfExpr>>,
    )>,
}

#[derive(Clone, Debug, Serialize)]
pub enum IfCondition {
    Expr(Box<Expr>),
    Let {
        let_token: LetToken,
        lhs: Box<Pattern>,
        eq_token: EqToken,
        rhs: Box<Expr>,
    },
}

#[derive(Clone, Debug, Serialize)]
pub enum LoopControlFlow<B, C = ()> {
    Continue(C),
    Break(B),
}

impl Spanned for IfExpr {
    fn span(&self) -> Span {
        let start = self.if_token.span();
        let end = match &self.else_opt {
            Some((_else_token, tail)) => match tail {
                LoopControlFlow::Break(block) => block.span(),
                LoopControlFlow::Continue(if_expr) => if_expr.span(),
            },
            None => self.then_block.span(),
        };
        Span::join(start, &end)
    }
}

#[derive(Clone, Debug, Serialize)]
pub enum ExprTupleDescriptor {
    Nil,
    Cons {
        head: Box<Expr>,
        comma_token: CommaToken,
        tail: Punctuated<Expr, CommaToken>,
    },
}

#[derive(Clone, Debug, Serialize)]
pub enum ExprArrayDescriptor {
    Sequence(Punctuated<Expr, CommaToken>),
    Repeat {
        value: Box<Expr>,
        semicolon_token: SemicolonToken,
        length: Box<Expr>,
    },
}

#[derive(Clone, Debug, Serialize)]
pub struct MatchBranch {
    pub pattern: Pattern,
    pub fat_right_arrow_token: FatRightArrowToken,
    pub kind: MatchBranchKind,
}

impl Spanned for MatchBranch {
    fn span(&self) -> Span {
        Span::join(self.pattern.span(), &self.kind.span())
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize)]
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
                Some(comma_token) => Span::join(block.span(), &comma_token.span()),
                None => block.span(),
            },
            MatchBranchKind::Expr { expr, comma_token } => {
                Span::join(expr.span(), &comma_token.span())
            }
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CodeBlockContents {
    pub statements: Vec<Statement>,
    pub final_expr_opt: Option<Box<Expr>>,
    pub span: Span,
}

impl Spanned for CodeBlockContents {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ExprStructField {
    pub field_name: Ident,
    pub expr_opt: Option<(ColonToken, Box<Expr>)>,
}

impl Spanned for ExprStructField {
    fn span(&self) -> Span {
        match &self.expr_opt {
            None => self.field_name.span(),
            Some((_colon_token, expr)) => Span::join(self.field_name.span(), &expr.span()),
        }
    }
}

impl Expr {
    /// Returns the resulting [Assignable] if the `self` is a
    /// valid [Assignable], or an error containing the [Expr]
    /// which causes the `self` to be an invalid [Assignable].
    ///
    /// In case of an error, the returned [Expr] can be `self`
    /// or any subexpression of `self` that is not allowed
    /// in assignment targets.
    #[allow(clippy::result_large_err)]
    pub fn try_into_assignable(self) -> Result<Assignable, Expr> {
        if let Expr::Deref { star_token, expr } = self {
            Ok(Assignable::Deref { star_token, expr })
        } else {
            Ok(Assignable::ElementAccess(
                self.try_into_element_access(false)?,
            ))
        }
    }

    #[allow(clippy::result_large_err)]
    fn try_into_element_access(
        self,
        accept_deref_without_parens: bool,
    ) -> Result<ElementAccess, Expr> {
        match self.clone() {
            Expr::Path(path_expr) => match path_expr.try_into_ident() {
                Ok(name) => Ok(ElementAccess::Var(name)),
                Err(path_expr) => Err(Expr::Path(path_expr)),
            },
            Expr::Index { target, arg } => match target.try_into_element_access(false) {
                Ok(target) => Ok(ElementAccess::Index {
                    target: Box::new(target),
                    arg,
                }),
                error => error,
            },
            Expr::FieldProjection {
                target,
                dot_token,
                name,
            } => match target.try_into_element_access(false) {
                Ok(target) => Ok(ElementAccess::FieldProjection {
                    target: Box::new(target),
                    dot_token,
                    name,
                }),
                error => error,
            },
            Expr::TupleFieldProjection {
                target,
                dot_token,
                field,
                field_span,
            } => match target.try_into_element_access(false) {
                Ok(target) => Ok(ElementAccess::TupleFieldProjection {
                    target: Box::new(target),
                    dot_token,
                    field,
                    field_span,
                }),
                error => error,
            },
            Expr::Parens(Parens { inner, .. }) => {
                if let Expr::Deref { expr, star_token } = *inner {
                    match expr.try_into_element_access(true) {
                        Ok(target) => Ok(ElementAccess::Deref {
                            target: Box::new(target),
                            star_token,
                            is_root_element: true,
                        }),
                        error => error,
                    }
                } else {
                    Err(self)
                }
            }
            Expr::Deref { expr, star_token } if accept_deref_without_parens => {
                match expr.try_into_element_access(true) {
                    Ok(target) => Ok(ElementAccess::Deref {
                        target: Box::new(target),
                        star_token,
                        is_root_element: false,
                    }),
                    error => error,
                }
            }
            expr => Err(expr),
        }
    }

    pub fn is_control_flow(&self) -> bool {
        match self {
            Expr::Block(..)
            | Expr::Asm(..)
            | Expr::If(..)
            | Expr::Match { .. }
            | Expr::While { .. }
            | Expr::For { .. } => true,
            Expr::Error(..)
            | Expr::Path(..)
            | Expr::Literal(..)
            | Expr::AbiCast { .. }
            | Expr::Struct { .. }
            | Expr::Tuple(..)
            | Expr::Parens(..)
            | Expr::Array(..)
            | Expr::Return { .. }
            | Expr::Panic { .. }
            | Expr::FuncApp { .. }
            | Expr::Index { .. }
            | Expr::MethodCall { .. }
            | Expr::FieldProjection { .. }
            | Expr::TupleFieldProjection { .. }
            | Expr::Ref { .. }
            | Expr::Deref { .. }
            | Expr::Not { .. }
            | Expr::Mul { .. }
            | Expr::Div { .. }
            | Expr::Pow { .. }
            | Expr::Modulo { .. }
            | Expr::Add { .. }
            | Expr::Sub { .. }
            | Expr::Shl { .. }
            | Expr::Shr { .. }
            | Expr::BitAnd { .. }
            | Expr::BitXor { .. }
            | Expr::BitOr { .. }
            | Expr::Equal { .. }
            | Expr::NotEqual { .. }
            | Expr::LessThan { .. }
            | Expr::GreaterThan { .. }
            | Expr::LessThanEq { .. }
            | Expr::GreaterThanEq { .. }
            | Expr::LogicalAnd { .. }
            | Expr::LogicalOr { .. }
            | Expr::Reassignment { .. }
            | Expr::Break { .. }
            | Expr::Continue { .. } => false,
        }
    }

    /// Friendly [Expr] name string used for error reporting,
    pub fn friendly_name(&self) -> &'static str {
        match self {
            Expr::Error(_, _) => "error",
            Expr::Path(_) => "path",
            Expr::Literal(_) => "literal",
            Expr::AbiCast { .. } => "ABI cast",
            Expr::Struct { .. } => "struct instantiation",
            Expr::Tuple(_) => "tuple",
            Expr::Parens(_) => "parentheses", // Note the plural!
            Expr::Block(_) => "block",
            Expr::Array(_) => "array",
            Expr::Asm(_) => "assembly block",
            Expr::Return { .. } => "return",
            Expr::Panic { .. } => "panic",
            Expr::If(_) => "if expression",
            Expr::Match { .. } => "match expression",
            Expr::While { .. } => "while loop",
            Expr::For { .. } => "for loop",
            Expr::FuncApp { .. } => "function call",
            Expr::Index { .. } => "array element access",
            Expr::MethodCall { .. } => "method call",
            Expr::FieldProjection { .. } => "struct field access",
            Expr::TupleFieldProjection { .. } => "tuple element access",
            Expr::Ref { .. } => "referencing",
            Expr::Deref { .. } => "dereferencing",
            Expr::Not { .. } => "negation",
            Expr::Mul { .. } => "multiplication",
            Expr::Div { .. } => "division",
            Expr::Pow { .. } => "power operation",
            Expr::Modulo { .. } => "modulo operation",
            Expr::Add { .. } => "addition",
            Expr::Sub { .. } => "subtraction",
            Expr::Shl { .. } => "left shift",
            Expr::Shr { .. } => "right shift",
            Expr::BitAnd { .. } => "bitwise and",
            Expr::BitXor { .. } => "bitwise xor",
            Expr::BitOr { .. } => "bitwise or",
            Expr::Equal { .. } => "equality",
            Expr::NotEqual { .. } => "non equality",
            Expr::LessThan { .. } => "less than operation",
            Expr::GreaterThan { .. } => "greater than operation",
            Expr::LessThanEq { .. } => "less than or equal operation",
            Expr::GreaterThanEq { .. } => "greater than or equal operation",
            Expr::LogicalAnd { .. } => "logical and",
            Expr::LogicalOr { .. } => "logical or",
            Expr::Reassignment { .. } => "reassignment",
            Expr::Break { .. } => "break",
            Expr::Continue { .. } => "continue",
        }
    }
}
