use crate::{Parse, ParseResult, Parser};

use sway_ast::submodule::Submodule;

impl Parse for Submodule {
    fn parse(parser: &mut Parser) -> ParseResult<Submodule> {
        let mod_token = parser.parse()?;
        let name = parser.parse()?;
        let semicolon_token = parser.parse()?;
        Ok(Submodule {
            mod_token,
            name,
            semicolon_token,
        })
    }
}
