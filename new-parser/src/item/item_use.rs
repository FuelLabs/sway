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
        match &self.visibility {
            Some(pub_token) => Span::join(pub_token.span(), self.semicolon_token.span()),
            None => Span::join(self.use_token.span(), self.semicolon_token.span()),
        }
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

    or! {
        group,
        glob,
        path,
        rename,
        name,
    }
    .try_map_with_span(|use_tree_opt: Option<UseTree>, span| {
        use_tree_opt.ok_or_else(|| ParseError::MalformedImport { span })
    })
}

pub fn item_use() -> impl Parser<Output = ItemUse> + Clone {
    pub_token()
    .then_whitespace()
    .optional()
    .then(use_token())
    .then_whitespace()
    .then(double_colon_token().then_optional_whitespace().optional())
    .then(use_tree())
    .then_optional_whitespace()
    .then(semicolon_token())
    .map(|((((visibility, use_token), root_import), tree), semicolon_token)| {
        ItemUse { visibility, use_token, root_import, tree, semicolon_token }
    })
}

