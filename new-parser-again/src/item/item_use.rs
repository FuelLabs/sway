use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemUse {
    pub visibility: Option<PubToken>,
    pub use_token: UseToken,
    pub root_import: Option<DoubleColonToken>,
    pub tree: UseTree,
    pub semicolon_token: SemicolonToken,
}

#[derive(Clone, Debug)]
pub enum UseTree {
    Group {
        imports: Braces<Punctuated<UseTree, CommaToken>>,
    },
    Name {
        name: Ident,
    },
    Rename {
        name: Ident,
        as_token: AsToken,
        alias: Ident,
    },
    Glob {
        star_token: StarToken,
    },
    Path {
        prefix: Ident,
        double_colon_token: DoubleColonToken,
        suffix: Box<UseTree>,
    },
}

impl Parse for UseTree {
    fn parse(parser: &mut Parser) -> ParseResult<UseTree> {
        if let Some(imports) = Braces::try_parse(parser)? {
            return Ok(UseTree::Group { imports });
        }
        if let Some(star_token) = parser.take() {
            return Ok(UseTree::Glob { star_token });
        }
        let name = match parser.take() {
            Some(name) => name,
            None => return Err(parser.emit_error("expected an import")),
        };
        if let Some(as_token) = parser.take() {
            let alias = parser.parse()?;
            return Ok(UseTree::Rename { name, as_token, alias });
        }
        if let Some(double_colon_token) = parser.take() {
            let suffix = parser.parse()?;
            return Ok(UseTree::Path { prefix: name, double_colon_token, suffix });
        }
        Ok(UseTree::Name { name })
    }
}

impl Parse for ItemUse {
    fn parse(parser: &mut Parser) -> ParseResult<ItemUse> {
        let visibility = parser.take();
        let use_token = parser.parse()?;
        let root_import = parser.take();
        let tree = parser.parse()?;
        let semicolon_token = parser.parse()?;
        Ok(ItemUse { visibility, use_token, root_import, tree, semicolon_token })
    }
}

