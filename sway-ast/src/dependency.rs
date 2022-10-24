use crate::priv_prelude::*;

pub struct Dependency {
    pub dep_token:       DepToken,
    pub path:            DependencyPath,
    pub semicolon_token: SemicolonToken,
}

impl Spanned for Dependency {
    fn span(&self) -> Span {
        Span::join(self.dep_token.span(), self.semicolon_token.span())
    }
}

pub struct DependencyPath {
    pub prefix:   Ident,
    pub suffixes: Vec<(ForwardSlashToken, Ident)>,
}

impl Spanned for DependencyPath {
    fn span(&self) -> Span {
        match self.suffixes.last() {
            Some((_forward_slash_token, suffix)) => Span::join(self.prefix.span(), suffix.span()),
            None => self.prefix.span(),
        }
    }
}
