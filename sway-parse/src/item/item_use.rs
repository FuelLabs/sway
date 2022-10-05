use crate::{Parse, ParseBracket, ParseErrorKind, ParseResult, Parser};

use sway_ast::{Braces, ItemUse, UseTree};

impl Parse for UseTree {
    fn parse(parser: &mut Parser) -> ParseResult<UseTree> {
        if let Some(imports) = Braces::try_parse(parser)? {
            return Ok(UseTree::Group { imports });
        }
        if let Some(star_token) = parser.take() {
            return Ok(UseTree::Glob { star_token });
        }
        if let Some(storage_token_1) = parser.take() {
            dbg!("are we not here?");
            let dot_token_1 = parser.parse()?;
            let name = parser.parse()?;
            let as_token = parser.parse()?;
            let storage_token_2 = parser.parse()?;
            let dot_token_2 = parser.parse()?;
            let alias = parser.parse()?;
            return Ok(dbg!(UseTree::StorageName {
                storage_token_1,
                dot_token_1,
                name,
                as_token,
                storage_token_2,
                dot_token_2,
                alias
            }));
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
            return Ok(dbg!(UseTree::Path {
                prefix: name,
                double_colon_token,
                suffix,
            }));
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
