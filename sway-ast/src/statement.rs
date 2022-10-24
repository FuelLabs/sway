use crate::priv_prelude::*;

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum Statement {
    Let(StatementLet),
    Item(Item),
    Expr {
        expr:                Expr,
        semicolon_token_opt: Option<SemicolonToken>,
    },
}

#[derive(Clone, Debug)]
pub struct StatementLet {
    pub let_token:       LetToken,
    pub pattern:         Pattern,
    pub ty_opt:          Option<(ColonToken, Ty)>,
    pub eq_token:        EqToken,
    pub expr:            Expr,
    pub semicolon_token: SemicolonToken,
}

impl Spanned for Statement {
    fn span(&self) -> Span {
        match self {
            Statement::Let(statement_let) => statement_let.span(),
            Statement::Item(item) => item.span(),
            Statement::Expr {
                expr,
                semicolon_token_opt,
            } => match semicolon_token_opt {
                None => expr.span(),
                Some(semicolon_token) => Span::join(expr.span(), semicolon_token.span()),
            },
        }
    }
}

impl Spanned for StatementLet {
    fn span(&self) -> Span {
        Span::join(self.let_token.span(), self.semicolon_token.span())
    }
}
