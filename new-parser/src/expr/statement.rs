use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub enum Statement {
    Let(StatementLet),
    Item(Item),
    Expr {
        expr: Expr,
        semicolon_token: SemicolonToken,
    },
}

#[derive(Clone, Debug)]
pub struct StatementLet {
    pub let_token: LetToken,
    pub pattern: Pattern,
    pub ty: Option<(ColonToken, Ty)>,
    pub eq_token: EqToken,
    pub expr: Expr,
    pub semicolon_token: SemicolonToken,
}

impl Spanned for Statement {
    fn span(&self) -> Span {
        match self {
            Statement::Let(statement_let) => statement_let.span(),
            Statement::Item(item) => item.span(),
            Statement::Expr { expr, semicolon_token } => {
                Span::join(expr.span(), semicolon_token.span())
            },
        }
    }
}

impl Spanned for StatementLet {
    fn span(&self) -> Span {
        Span::join(self.let_token.span(), self.semicolon_token.span())
    }
}

pub fn statement() -> impl Parser<Output = Statement> + Clone {
    let statement_let = {
        statement_let()
        .map(Statement::Let)
    };
    let item = {
        item()
        .map(Statement::Item)
    };
    let expr = {
        lazy(|| expr())
        .then(semicolon_token())
        .map(|(expr, semicolon_token)| Statement::Expr { expr, semicolon_token })
    };

    statement_let
    .or(item)
    .or(expr)
}

pub fn statement_let() -> impl Parser<Output = StatementLet> + Clone {
    let_token()
    .then_whitespace()
    .then(pattern())
    .then_optional_whitespace()
    .then(
        colon_token()
        .then_optional_whitespace()
        .then(ty())
        .then_optional_whitespace()
        .optional()
    )
    .then(eq_token())
    .then_optional_whitespace()
    .then(lazy(|| expr()))
    .then_optional_whitespace()
    .then(semicolon_token())
    .map(|(((((let_token, pattern), ty), eq_token), expr), semicolon_token)| {
        StatementLet { let_token, pattern, ty, eq_token, expr, semicolon_token }
    })
}

