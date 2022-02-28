use crate::priv_prelude::*;

pub struct Dependency {
    pub dep_token: DepToken,
    pub path: DependencyPath,
    pub semicolon_token: SemicolonToken,
}

pub struct DependencyPath {
    pub prefix: Ident,
    pub suffixes: Vec<(ForwardSlashToken, Ident)>,
}

impl Parse for DependencyPath {
    fn parse(parser: &mut Parser) -> ParseResult<DependencyPath> {
        let prefix = parser.parse()?;
        let mut suffixes = Vec::new();
        loop {
            let forward_slash_token = match parser.take() {
                Some(forward_slash_token) => forward_slash_token,
                None => break,
            };
            let suffix = parser.parse()?;
            suffixes.push((forward_slash_token, suffix));
        }
        Ok(DependencyPath {
            prefix,
            suffixes,
        })
    }
}

impl Parse for Dependency {
    fn parse(parser: &mut Parser) -> ParseResult<Dependency> {
        let dep_token = parser.parse()?;
        let path = parser.parse()?;
        let semicolon_token = parser.parse()?;
        Ok(Dependency { dep_token, path, semicolon_token })
    }
}

