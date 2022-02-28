use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct GenericParams {
    pub parameters: AngleBrackets<Punctuated<Ident, CommaToken>>,
}

impl Parse for GenericParams {
    fn parse(parser: &mut Parser) -> ParseResult<GenericParams> {
        let less_than_token = parser.parse()?;
        let mut value_separator_pairs = Vec::new();
        let (final_value_opt, greater_than_token) = loop {
            if let Some(greater_than_token) = parser.take() {
                break (None, greater_than_token);
            };
            let ident = parser.parse()?;
            if let Some(greater_than_token) = parser.take() {
                break (Some(Box::new(ident)), greater_than_token);
            };
            let comma_token = parser.parse()?;
            value_separator_pairs.push((ident, comma_token));
        };
        let punctuated = Punctuated { value_separator_pairs, final_value_opt };
        let parameters = AngleBrackets { less_than_token, inner: punctuated, greater_than_token };
        Ok(GenericParams { parameters })
    }
}

#[derive(Clone, Debug)]
pub struct GenericArgs {
    pub parameters: AngleBrackets<Punctuated<Ty, CommaToken>>,
}

impl Parse for GenericArgs {
    fn parse(parser: &mut Parser) -> ParseResult<GenericArgs> {
        let less_than_token = parser.parse()?;
        let mut value_separator_pairs = Vec::new();
        let (final_value_opt, greater_than_token) = loop {
            if let Some(greater_than_token) = parser.take() {
                break (None, greater_than_token);
            };
            let ty = parser.parse()?;
            if let Some(greater_than_token) = parser.take() {
                break (Some(Box::new(ty)), greater_than_token);
            };
            let comma_token = parser.parse()?;
            value_separator_pairs.push((ty, comma_token));
        };
        let punctuated = Punctuated { value_separator_pairs, final_value_opt };
        let parameters = AngleBrackets { less_than_token, inner: punctuated, greater_than_token };
        Ok(GenericArgs { parameters })
    }
}
