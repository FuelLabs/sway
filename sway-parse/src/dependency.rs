use crate::{Parse, ParseResult, Parser};

use sway_ast::dependency::{Dependency, DependencyPath};

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
