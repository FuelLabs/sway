use crate::{Parse, ParseResult, Parser};

use sway_ast::{ConfigurableField, ItemConfigurable};

impl Parse for ConfigurableField {
    fn parse(parser: &mut Parser) -> ParseResult<ConfigurableField> {
        let name = parser.parse()?;
        let colon_token = parser.parse()?;
        let ty = parser.parse()?;
        let eq_token = parser.parse()?;
        let initializer = parser.parse()?;
        Ok(ConfigurableField {
            name,
            colon_token,
            ty,
            eq_token,
            initializer,
        })
    }
}

impl Parse for ItemConfigurable {
    fn parse(parser: &mut Parser) -> ParseResult<ItemConfigurable> {
        let configurable_token = parser.parse()?;
        let fields = parser.parse()?;
        Ok(ItemConfigurable {
            configurable_token,
            fields,
        })
    }
}
