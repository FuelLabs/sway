use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct PatternStruct {
    path: PathExpr,
    fields: Braces<Punctuated<PatternStructField, CommaToken>>,
}

#[derive(Clone, Debug)]
pub struct PatternStructField {
    pub field_name: Ident,
    pub pattern_opt: Option<(ColonToken, Box<Pattern>)>,
}

impl Spanned for PatternStruct {
    fn span(&self) -> Span {
        Span::join(self.path.span(), self.fields.span())
    }
}

impl Spanned for PatternStructField {
    fn span(&self) -> Span {
        match &self.pattern_opt {
            Some((_, pattern)) => Span::join(self.field_name.span(), pattern.span()),
            None => self.field_name.span(),
        }
    }
}

pub fn pattern_struct() -> impl Parser<Output = PatternStruct> + Clone {
    path_expr()
    .then_optional_whitespace()
    .then(braces(padded(punctuated(pattern_struct_field(), padded(comma_token())))))
    .map(|(path, fields)| {
        PatternStruct { path, fields }
    })
}

pub fn pattern_struct_field() -> impl Parser<Output = PatternStructField> + Clone {
    ident()
    .then(optional_leading_whitespace(
        colon_token()
        .then_optional_whitespace()
        .then(lazy(|| pattern()).map(Box::new))
        .optional()
    ))
    .map(|(field_name, pattern_opt)| {
        PatternStructField { field_name, pattern_opt }
    })
}

