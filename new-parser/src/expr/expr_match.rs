use crate::priv_prelude::*;

#[derive(Debug, Clone)]
pub struct ExprMatch {
    pub match_token: MatchToken,
    pub condition: Box<Expr>,
    pub branches: Braces<Vec<MatchBranch>>,
}

#[derive(Debug, Clone)]
pub struct MatchBranch {
    pub pattern: Pattern,
    pub fat_right_arrow_token: FatRightArrowToken,
    pub kind: MatchBranchKind,
}

#[derive(Debug, Clone)]
pub enum MatchBranchKind {
    Block(CodeBlock),
    Expr {
        expr: Expr,
        comma_token: CommaToken,
    },
}

impl Spanned for ExprMatch {
    fn span(&self) -> Span {
        Span::join(self.match_token.span(), self.branches.span())
    }
}

impl Spanned for MatchBranch {
    fn span(&self) -> Span {
        Span::join(self.pattern.span(), self.kind.span())
    }
}

impl Spanned for MatchBranchKind {
    fn span(&self) -> Span {
        match self {
            MatchBranchKind::Block(code_block) => code_block.span(),
            MatchBranchKind::Expr { expr, comma_token } => {
                Span::join(expr.span(), comma_token.span())
            },
        }
    }
}

pub fn expr_match() -> impl Parser<Output = ExprMatch> + Clone {
    match_token()
    .then_optional_whitespace()
    .commit()
    .then(lazy(|| expr()).map(Box::new))
    .then_optional_whitespace()
    .then(braces(
        optional_leading_whitespace(match_branch())
        .repeated()
        .then_optional_whitespace()
    ))
    .map(|((match_token, condition), branches)| {
        ExprMatch { match_token, condition, branches }
    })
}

pub fn match_branch() -> impl Parser<Output = MatchBranch> + Clone {
    pattern()
    .then_optional_whitespace()
    .commit()
    .then(fat_right_arrow_token())
    .then_optional_whitespace()
    .then(match_branch_kind())
    .map(|((pattern, fat_right_arrow_token), kind)| {
        MatchBranch { pattern, fat_right_arrow_token, kind }
    })
}

pub fn match_branch_kind() -> impl Parser<Output = MatchBranchKind> + Clone {
    let block = {
        code_block()
        .map(|code_block| MatchBranchKind::Block(code_block))
    };
    let expr = {
        lazy(|| expr())
        .then_optional_whitespace()
        .then(comma_token())
        .map(|(expr, comma_token)| MatchBranchKind::Expr { expr, comma_token })
    };

    expr.or(block)
}

