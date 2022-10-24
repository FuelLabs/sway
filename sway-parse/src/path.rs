use crate::{Parse, ParseResult, Parser};

use sway_ast::{
    keywords::{DoubleColonToken, OpenAngleBracketToken, SelfToken, StorageToken},
    AngleBrackets, PathExpr, PathExprSegment, PathType, PathTypeSegment, QualifiedPathRoot,
};
use sway_types::Ident;

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
        let mut suffix = Vec::new();
        while let Some(double_colon_token) = parser.take() {
            let segment = parser.parse()?;
            suffix.push((double_colon_token, segment));
        }
        Ok(PathExpr {
            root_opt,
            prefix,
            suffix,
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
            fully_qualified: parser.take(),
            name:            parse_ident(parser)?,
            generics_opt:    parser
                .guarded_parse::<(DoubleColonToken, OpenAngleBracketToken), _>()?,
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
        let fully_qualified = parser.take();
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
        Ok(PathTypeSegment {
            fully_qualified,
            name,
            generics_opt,
        })
    }
}

impl Parse for QualifiedPathRoot {
    fn parse(parser: &mut Parser) -> ParseResult<QualifiedPathRoot> {
        let ty = parser.parse()?;
        let as_trait = match parser.take() {
            Some(as_token) => {
                let path_type = parser.parse()?;
                Some((as_token, path_type))
            }
            None => None,
        };
        Ok(QualifiedPathRoot { ty, as_trait })
    }
}
