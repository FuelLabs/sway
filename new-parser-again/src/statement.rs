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
    pub ty_opt: Option<(ColonToken, Ty)>,
    pub eq_token: EqToken,
    pub expr: Expr,
    pub semicolon_token: SemicolonToken,
}

