use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct GenericParams {
    pub parameters: AngleBrackets<Punctuated<Ident, CommaToken>>,
}

impl Parse for GenericParams {
    fn parse(parser: &mut Parser) -> ParseResult<GenericParams> {
        let open_angle_bracket_token = parser.parse()?;
        let mut value_separator_pairs = Vec::new();
        let (final_value_opt, close_angle_bracket_token) = loop {
            if let Some(close_angle_bracket_token) = parser.take() {
                break (None, close_angle_bracket_token);
            };
            let ident = parser.parse()?;
            if let Some(close_angle_bracket_token) = parser.take() {
                break (Some(Box::new(ident)), close_angle_bracket_token);
            };
            let comma_token = parser.parse()?;
            value_separator_pairs.push((ident, comma_token));
        };
        let punctuated = Punctuated {
            value_separator_pairs,
            final_value_opt,
        };
        let parameters = AngleBrackets {
            open_angle_bracket_token,
            inner: punctuated,
            close_angle_bracket_token,
        };
        Ok(GenericParams { parameters })
    }
}

#[derive(Clone, Debug)]
pub struct GenericArgs {
    pub parameters: AngleBrackets<Punctuated<Ty, CommaToken>>,
}

impl GenericArgs {
    pub fn span(&self) -> Span {
        self.parameters.span()
    }
}

impl Parse for GenericArgs {
    fn parse(parser: &mut Parser) -> ParseResult<GenericArgs> {
        let open_angle_bracket_token = parser.parse()?;
        let mut value_separator_pairs = Vec::new();
        let (final_value_opt, close_angle_bracket_token) = loop {
            if let Some(close_angle_bracket_token) = parser.take() {
                break (None, close_angle_bracket_token);
            };
            let ty = parser.parse()?;
            if let Some(close_angle_bracket_token) = parser.take() {
                break (Some(Box::new(ty)), close_angle_bracket_token);
            };
            let comma_token = parser.parse()?;
            value_separator_pairs.push((ty, comma_token));
        };
        let punctuated = Punctuated {
            value_separator_pairs,
            final_value_opt,
        };
        let parameters = AngleBrackets {
            open_angle_bracket_token,
            inner: punctuated,
            close_angle_bracket_token,
        };
        Ok(GenericArgs { parameters })
    }
}
