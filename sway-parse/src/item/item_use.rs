use crate::{Parse, ParseBracket, ParseResult, Parser};

use sway_ast::{Braces, ItemUse, UseTree};
use sway_error::parser_error::ParseErrorKind;

impl Parse for UseTree {
    fn parse(parser: &mut Parser) -> ParseResult<UseTree> {
        if let Some(imports) = Braces::try_parse(parser)? {
            return Ok(UseTree::Group { imports });
        }
        if let Some(star_token) = parser.take() {
            return Ok(UseTree::Glob { star_token });
        }
        let name = parser
            .take()
            .ok_or_else(|| parser.emit_error(ParseErrorKind::ExpectedImportNameGroupOrGlob))?;
        if let Some(as_token) = parser.take() {
            let alias = parser.parse()?;
            return Ok(UseTree::Rename {
                name,
                as_token,
                alias,
            });
        }
        if let Some(double_colon_token) = parser.take() {
            let suffix = parser.parse()?;
            return Ok(UseTree::Path {
                prefix: name,
                double_colon_token,
                suffix,
            });
        }
        Ok(UseTree::Name { name })
    }
}

impl Parse for ItemUse {
    fn parse(parser: &mut Parser) -> ParseResult<ItemUse> {
        let use_token = parser.parse()?;
        let root_import = parser.take();
        let tree = parser.parse()?;
        let semicolon_token = parser.parse()?;
        Ok(ItemUse {
            visibility: None,
            use_token,
            root_import,
            tree,
            semicolon_token,
        })
    }
}
