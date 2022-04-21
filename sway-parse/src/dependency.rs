use crate::priv_prelude::*;

pub struct Dependency {
    pub dep_token: DepToken,
    pub path: DependencyPath,
    pub semicolon_token: SemicolonToken,
}

impl Dependency {
    pub fn span(&self) -> Span {
        Span::join(self.dep_token.span(), self.semicolon_token.span())
    }
}

pub struct DependencyPath {
    pub prefix: Ident,
    pub suffixes: Vec<(ForwardSlashToken, Ident)>,
}

impl DependencyPath {
    pub fn span(&self) -> Span {
        match self.suffixes.last() {
            Some((_forward_slash_token, suffix)) => {
                Span::join(self.prefix.span().clone(), suffix.span().clone())
            }
            None => self.prefix.span().clone(),
        }
    }
}

impl Parse for DependencyPath {
    fn parse(parser: &mut Parser) -> ParseResult<DependencyPath> {
        let prefix = parser.parse()?;
        let mut suffixes = Vec::new();
        while let Some(forward_slash_token) = parser.take() {
            let suffix = parser.parse()?;
            suffixes.push((forward_slash_token, suffix));
        }
        Ok(DependencyPath { prefix, suffixes })
    }
}

impl Parse for Dependency {
    fn parse(parser: &mut Parser) -> ParseResult<Dependency> {
        let dep_token = parser.parse()?;
        let path = parser.parse()?;
        let semicolon_token = parser.parse()?;
        Ok(Dependency {
            dep_token,
            path,
            semicolon_token,
        })
    }
}
