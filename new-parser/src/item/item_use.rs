use crate::priv_prelude::*;

pub struct ItemUse {
    pub use_token: UseToken,
    pub root_import: Option<DoubleColonToken>,
    pub tree: UseTree,
    pub semicolon_token: SemicolonToken,
}

impl Spanned for ItemUse {
    fn span(&self) -> Span {
        Span::join(self.use_token.span(), self.semicolon_token.span())
    }
}

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

impl Spanned for UseTree {
    fn span(&self) -> Span {
        match self {
            UseTree::Group { imports } => imports.span(),
            UseTree::Name { name } => name.span(),
            UseTree::Rename { name, alias, .. } => {
                Span::join(name.span(), alias.span())
            },
            UseTree::Glob { star_token } => star_token.span(),
            UseTree::Path { prefix, suffix, .. } => {
                Span::join(prefix.span(), suffix.span())
            },
        }
    }
}

pub fn use_tree() -> impl Parser<Output = UseTree> + Clone {
    let group = {
        braces(punctuated(lazy(|| use_tree()), comma_token()))
        .map(|imports| UseTree::Group { imports })
    };
    let name = {
        ident()
        .map(|name| UseTree::Name { name })
    };
    let rename = {
        ident()
        .then_whitespace()
        .then(as_token())
        .then_whitespace()
        .then(ident())
        .map(|((name, as_token), alias)| UseTree::Rename { name, as_token, alias })
    };
    let glob = {
        star_token()
        .map(|star_token| UseTree::Glob { star_token })
    };
    let path = {
        ident()
        .then_optional_whitespace()
        .then(double_colon_token())
        .then_optional_whitespace()
        .then(lazy(|| use_tree()))
        .map(|((prefix, double_colon_token), suffix)| UseTree::Path {
            prefix,
            double_colon_token, 
            suffix: Box::new(suffix),
        })
    };

    group
    .or(glob)
    .or(path)
    .or(rename)
    .or(name)
}

pub fn item_use() -> impl Parser<Output = ItemUse> + Clone {
    use_token()
    .then_whitespace()
    .then(double_colon_token().then_optional_whitespace().optional())
    .then(use_tree())
    .then_optional_whitespace()
    .then(semicolon_token())
    .map(|
         (((use_token, root_import), tree), semicolon_token): (((_, Result<_, _>), _), _)
    | {
        ItemUse {
            use_token,
            root_import: root_import.ok(),
            tree,
            semicolon_token,
        }
    })
}

