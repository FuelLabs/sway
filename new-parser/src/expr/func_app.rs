use crate::priv_prelude::*;

pub struct ExprFuncApp {
    pub func: Box<Expr>,
    pub args: Parens<Punctuated<Expr, CommaToken>>,
}

impl Spanned for ExprFuncApp {
    fn span(&self) -> Span {
        Span::join(self.func.span(), self.args.span())
    }
}

