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

impl Parse for WhereClause {
    fn parse(parser: &mut Parser) -> ParseResult<WhereClause> {
        let where_token = parser.parse()?;
        let mut value_separator_pairs = Vec::new();
        let final_value_opt = loop {
            let ty_name = match parser.take() {
                Some(ty_name) => ty_name,
                None => break None,
            };
            let colon_token = parser.parse()?;
            let bounds = parser.parse()?;
            let where_bound = WhereBound {
                ty_name,
                colon_token,
                bounds,
            };
            match parser.take() {
                Some(comma_token) => value_separator_pairs.push((where_bound, comma_token)),
                None => break Some(Box::new(where_bound)),
            }
        };
        let bounds = Punctuated {
            value_separator_pairs,
            final_value_opt,
        };
        Ok(WhereClause {
            where_token,
            bounds,
        })
    }
}

impl WhereClause {
    pub fn span(&self) -> Span {
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

impl WhereBound {
    pub fn span(&self) -> Span {
        Span::join(self.ty_name.span().clone(), self.bounds.span())
    }
}
