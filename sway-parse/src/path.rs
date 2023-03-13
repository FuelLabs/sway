use crate::{Parse, ParseResult, Parser};

use sway_ast::keywords::{DoubleColonToken, OpenAngleBracketToken, SelfToken, StorageToken};
use sway_ast::{
    AngleBrackets, PathExpr, PathExprSegment, PathType, PathTypeSegment, QualifiedPathRoot,
};
use sway_types::{Ident, Spanned};

impl Parse for PathExpr {
    fn parse(parser: &mut Parser) -> ParseResult<PathExpr> {
        let root_opt = match parser.take() {
            Some(open_angle_bracket_token) => {
                let qualified_path_root = parser.parse()?;
                let close_angle_bracket_token = parser.parse()?;
                let angle_brackets = AngleBrackets {
                    open_angle_bracket_token,
                    inner: qualified_path_root,
                    close_angle_bracket_token,
                };
                let double_colon_token = parser.parse()?;
                Some((Some(angle_brackets), double_colon_token))
            }
            None => parser
                .take()
                .map(|double_colon_token| (None, double_colon_token)),
        };
        let prefix = parser.parse()?;
        let mut suffix: Vec<(DoubleColonToken, PathExprSegment)> = Vec::new();
        let mut incomplete_suffix = false;
        while let Some(double_colon_token) = parser.take() {
            if let Ok(segment) = parser.parse() {
                suffix.push((double_colon_token, segment));
            } else {
                incomplete_suffix = true;
                // this is to make the span be `foo::` instead of just `foo`
                let dummy_path_expr_segment = PathExprSegment {
                    name: Ident::new(double_colon_token.span()),
                    generics_opt: None,
                };
                suffix.push((double_colon_token, dummy_path_expr_segment));
                break;
            }
        }
        Ok(PathExpr {
            root_opt,
            prefix,
            suffix,
            incomplete_suffix,
        })
    }
}

fn parse_ident(parser: &mut Parser) -> ParseResult<Ident> {
    if let Some(token) = parser.take::<StorageToken>() {
        Ok(Ident::from(token))
    } else if let Some(token) = parser.take::<SelfToken>() {
        Ok(Ident::from(token))
    } else {
        parser.parse::<Ident>()
    }
}

impl Parse for PathExprSegment {
    fn parse(parser: &mut Parser) -> ParseResult<PathExprSegment> {
        Ok(PathExprSegment {
            name: parse_ident(parser)?,
            generics_opt: parser.guarded_parse::<(DoubleColonToken, OpenAngleBracketToken), _>()?,
        })
    }
}

impl Parse for PathType {
    fn parse(parser: &mut Parser) -> ParseResult<PathType> {
        let root_opt = match parser.take() {
            Some(open_angle_bracket_token) => {
                let qualified_path_root = parser.parse()?;
                let close_angle_bracket_token = parser.parse()?;
                let angle_brackets = AngleBrackets {
                    open_angle_bracket_token,
                    inner: qualified_path_root,
                    close_angle_bracket_token,
                };
                let double_colon_token = parser.parse()?;
                Some((Some(angle_brackets), double_colon_token))
            }
            None => parser
                .take()
                .map(|double_colon_token| (None, double_colon_token)),
        };
        let prefix = parser.parse()?;
        let mut suffix = Vec::new();
        while let Some(double_colon_token) = parser.take() {
            let segment = parser.parse()?;
            suffix.push((double_colon_token, segment));
        }
        Ok(PathType {
            root_opt,
            prefix,
            suffix,
        })
    }
}

impl Parse for PathTypeSegment {
    fn parse(parser: &mut Parser) -> ParseResult<PathTypeSegment> {
        let name = parse_ident(parser)?;
        let generics_opt =
            if let Some(generics) = parser.guarded_parse::<OpenAngleBracketToken, _>()? {
                Some((None, generics))
            } else if let Some((double_colon_token, generics)) =
                parser.guarded_parse::<(DoubleColonToken, OpenAngleBracketToken), _>()?
            {
                Some((Some(double_colon_token), generics))
            } else {
                None
            };
        Ok(PathTypeSegment { name, generics_opt })
    }
}

impl Parse for QualifiedPathRoot {
    fn parse(parser: &mut Parser) -> ParseResult<QualifiedPathRoot> {
        let ty = parser.parse()?;
        let as_trait = match parser.take() {
            Some(as_token) => Some((as_token, parser.parse()?)),
            None => None,
        };
        Ok(QualifiedPathRoot { ty, as_trait })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::parse;
    use insta::*;

    #[test]
    fn parse_nested_path() {
        assert_ron_snapshot!(parse::<PathExpr>(r#"
            std::vec::Vec
        "#,), @r###"
        PathExpr(
          root_opt: None,
          prefix: PathExprSegment(
            name: Ident(
              to_string: "std",
              span: (13, 16),
            ),
            generics_opt: None,
          ),
          suffix: [
            (DoubleColonToken(
              span: (16, 18),
            ), PathExprSegment(
              name: Ident(
                to_string: "vec",
                span: (18, 21),
              ),
              generics_opt: None,
            )),
            (DoubleColonToken(
              span: (21, 23),
            ), PathExprSegment(
              name: Ident(
                to_string: "Vec",
                span: (23, 26),
              ),
              generics_opt: None,
            )),
          ],
        )
        "###);
    }
}
