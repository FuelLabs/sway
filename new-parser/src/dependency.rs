use crate::priv_prelude::*;

pub struct Dependency {
    pub dep_token: DepToken,
    pub path: DependencyPath,
    pub semicolon_token: SemicolonToken,
}

pub struct DependencyPath {
    pub prefix: Ident,
    pub suffix: Vec<(ForwardSlashToken, Ident)>,
}

impl DependencyPath {
    pub fn iter(&self) -> impl Iterator<Item = &Ident> {
        iter::once(&self.prefix)
        .chain(self.suffix.iter().map(|(_, name)| name))
    }
}

impl Spanned for Dependency {
    fn span(&self) -> Span {
        Span::join(self.dep_token.span(), self.semicolon_token.span())
    }
}

impl Spanned for DependencyPath {
    fn span(&self) -> Span {
        match self.suffix.last() {
            None => self.prefix.span(),
            Some((_, name)) => {
                Span::join(self.prefix.span(), name.span())
            },
        }
    }
}

pub fn dependency() -> impl Parser<Output = Dependency> + Clone {
    dep_token()
    .then_whitespace()
    .then(dependency_path())
    .then_optional_whitespace()
    .then(semicolon_token())
    .map(|((dep_token, path), semicolon_token)| {
        Dependency { dep_token, path, semicolon_token }
    })
}

pub fn dependency_path() -> impl Parser<Output = DependencyPath> + Clone {
    ident()
    .then(
        forward_slash_token()
        .then(ident())
        .repeated()
    )
    .map(|(prefix, suffix)| {
        DependencyPath { prefix, suffix }
    })
}

