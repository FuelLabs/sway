use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ExprStruct {
    path: PathExpr,
    fields: Braces<Punctuated<ExprStructField, CommaToken>>,
}

#[derive(Clone, Debug)]
pub struct ExprStructField  {
    pub field_name: Ident,
    pub expr_opt: Option<(ColonToken, Box<Expr>)>,
}

impl Spanned for ExprStruct {
    fn span(&self) -> Span {
        Span::join(self.path.span(), self.fields.span())
    }
}

impl Spanned for ExprStructField {
    fn span(&self) -> Span {
        match &self.expr_opt {
            Some((_, expr)) => Span::join(self.field_name.span(), expr.span()),
            None => self.field_name.span(),
        }
    }
}

pub fn expr_struct() -> impl Parser<Output = ExprStruct> + Clone {
    path_expr()
    .then_optional_whitespace()
    .then(braces(padded(punctuated(expr_struct_field(), padded(comma_token())))))
    .map(|(path, fields)| {
        ExprStruct { path, fields }
    })
}

pub fn expr_struct_field() -> impl Parser<Output = ExprStructField> + Clone {
    ident()
    .then(optional_leading_whitespace(
        colon_token()
        .then_optional_whitespace()
        .then(lazy(|| expr()).map(Box::new))
        .optional()
    ))
    .map(|(field_name, expr_opt)| {
        ExprStructField { field_name, expr_opt }
    })
}

