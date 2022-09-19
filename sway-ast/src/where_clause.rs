use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct WhereClause {
    pub where_token: WhereToken,
    pub bounds: Punctuated<WhereBound, CommaToken>,
}

#[derive(Clone, Debug)]
pub struct WhereBound {
    pub ty_name: Ident,
    pub colon_token: ColonToken,
    pub bounds: Traits,
}

impl Spanned for WhereClause {
    fn span(&self) -> Span {
        let where_token_span = self.where_token.span();
        match &self.bounds.final_value_opt {
            Some(where_bound) => Span::join(where_token_span, where_bound.span()),
            None => match self.bounds.value_separator_pairs.last() {
                Some((_, comma_token)) => Span::join(where_token_span, comma_token.span()),
                None => where_token_span,
            },
        }
    }
}

impl Spanned for WhereBound {
    fn span(&self) -> Span {
        Span::join(self.ty_name.span(), self.bounds.span())
    }
}
