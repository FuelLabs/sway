use crate::{Parse, ParseResult, Parser};

use sway_ast::keywords::CommaToken;
use sway_ast::punctuated::Punctuated;
use sway_ast::{AngleBrackets, GenericArgs, GenericParams};

impl Parse for GenericParams {
    fn parse(parser: &mut Parser) -> ParseResult<GenericParams> {
        parse_angle_comma(parser).map(|parameters| GenericParams { parameters })
    }
}

impl Parse for GenericArgs {
    fn parse(parser: &mut Parser) -> ParseResult<GenericArgs> {
        parse_angle_comma(parser).map(|parameters| GenericArgs { parameters })
    }
}

/// Parse a list of `T`s delimited by `<` and `>` and separated by `,`.
fn parse_angle_comma<T: Parse>(
    parser: &mut Parser,
) -> ParseResult<AngleBrackets<Punctuated<T, CommaToken>>> {
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
    Ok(AngleBrackets {
        open_angle_bracket_token,
        inner: punctuated,
        close_angle_bracket_token,
    })
}
