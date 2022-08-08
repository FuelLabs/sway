use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemUse {
    pub visibility: Option<PubToken>,
    pub use_token: UseToken,
    pub root_import: Option<DoubleColonToken>,
    pub tree: UseTree,
    pub semicolon_token: SemicolonToken,
}

impl Spanned for ItemUse {
    fn span(&self) -> Span {
        let start = match &self.visibility {
            Some(pub_token) => pub_token.span(),
            None => self.use_token.span(),
        };
        let end = self.semicolon_token.span();
        Span::join(start, end)
    }
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
