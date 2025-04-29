use crate::{Parse, ParseBracket, ParseResult, ParseToEnd, Parser, ParserConsumed};
use sway_ast::brackets::{Parens, SquareBrackets};
use sway_ast::keywords::{DoubleColonToken, OpenAngleBracketToken, PtrToken, SliceToken};
use sway_ast::ty::{Ty, TyArrayDescriptor, TyTupleDescriptor};
use sway_ast::{Expr, Literal};
use sway_error::parser_error::ParseErrorKind;
use sway_types::{ast::Delimiter, Ident};

impl Parse for Ty {
    fn parse(parser: &mut Parser) -> ParseResult<Ty> {
        // parse parens carefully, such that only patterns of (ty) are parsed as ty,
        // and patterns of (ty,) are parsed as one-arity tuples with one element ty
        if let Some((mut parser, span)) = parser.enter_delimited(Delimiter::Parenthesis) {
            if let Some(_consumed) = parser.check_empty() {
                return Ok(Ty::Tuple(Parens::new(TyTupleDescriptor::Nil, span)));
            }
            let head = parser.parse()?;
            if let Some(comma_token) = parser.take() {
                let (tail, _consumed) = parser.parse_to_end()?;
                let tuple = TyTupleDescriptor::Cons {
                    head,
                    comma_token,
                    tail,
                };
                return Ok(Ty::Tuple(Parens::new(tuple, span)));
            }
            if parser.check_empty().is_some() {
                return Ok(*head);
            }
            return Err(parser
                .emit_error(ParseErrorKind::ExpectedCommaOrCloseParenInTupleOrParenExpression));
        }

        if let Some((mut inner_parser, span)) = parser.enter_delimited(Delimiter::Bracket) {
            // array like [type; len]
            if let Ok((array, _)) = inner_parser.try_parse_to_end::<TyArrayDescriptor>(false) {
                return Ok(Ty::Array(SquareBrackets { inner: array, span }));
            }

            // slice like [type]
            if let Ok(Some((ty, _))) = inner_parser.try_parse_and_check_empty::<Ty>(false) {
                return Ok(Ty::Slice {
                    slice_token: None,
                    ty: SquareBrackets {
                        inner: Box::new(ty),
                        span,
                    },
                });
            }
        }

        if let Some(str_token) = parser.take() {
            let length = SquareBrackets::try_parse_all_inner(parser, |mut parser| {
                parser.emit_error(ParseErrorKind::UnexpectedTokenAfterStrLength)
            })?;
            let t = match length {
                Some(length) => Ty::StringArray { str_token, length },
                None => Ty::StringSlice(str_token),
            };
            return Ok(t);
        }

        if let Some(underscore_token) = parser.take() {
            return Ok(Ty::Infer { underscore_token });
        }

        if let Some(ptr_token) = parser.take::<PtrToken>() {
            let ty = SquareBrackets::parse_all_inner(parser, |mut parser| {
                parser.emit_error(ParseErrorKind::UnexpectedTokenAfterPtrType)
            })?;
            return Ok(Ty::Ptr { ptr_token, ty });
        }

        // slice like __slice[type]
        // TODO: deprecate this syntax (see https://github.com/FuelLabs/sway/issues/5110)
        if let Some(slice_token) = parser.take::<SliceToken>() {
            let ty = SquareBrackets::<Box<Ty>>::parse_all_inner(parser, |mut parser| {
                parser.emit_error(ParseErrorKind::UnexpectedTokenAfterSliceType)
            })?;
            return Ok(Ty::Slice {
                slice_token: Some(slice_token),
                ty,
            });
        }

        if let Some(ampersand_token) = parser.take() {
            let mut_token = parser.take();
            let ty = Box::new(parser.parse()?);
            return Ok(Ty::Ref {
                ampersand_token,
                mut_token,
                ty,
            });
        }

        if let Some(bang_token) = parser.take() {
            return Ok(Ty::Never { bang_token });
        }

        if parser.peek::<OpenAngleBracketToken>().is_some()
            || parser.peek::<DoubleColonToken>().is_some()
            || parser.peek::<Ident>().is_some()
        {
            let path_type = parser.parse()?;
            return Ok(Ty::Path(path_type));
        }

        if let Ok(literal) = parser.parse::<Literal>() {
            return Ok(Ty::Expr(Box::new(Expr::Literal(literal))));
        }

        Err(parser.emit_error(ParseErrorKind::ExpectedType))
    }
}

impl ParseToEnd for TyArrayDescriptor {
    fn parse_to_end<'a, 'e>(
        mut parser: Parser<'a, '_>,
    ) -> ParseResult<(TyArrayDescriptor, ParserConsumed<'a>)> {
        let ty = parser.parse()?;
        let semicolon_token = parser.parse()?;
        let length = parser.parse()?;
        let consumed = match parser.check_empty() {
            Some(consumed) => consumed,
            None => {
                return Err(parser.emit_error(ParseErrorKind::UnexpectedTokenAfterArrayTypeLength))
            }
        };
        let descriptor = TyArrayDescriptor {
            ty,
            semicolon_token,
            length,
        };
        Ok((descriptor, consumed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::parse;
    use assert_matches::*;

    #[test]
    fn parse_ptr() {
        let item = parse::<Ty>(
            r#"
            __ptr[T]
            "#,
        );
        assert_matches!(item, Ty::Ptr { .. });
    }

    #[test]
    fn parse_array() {
        let item = parse::<Ty>("[T; 1]");
        assert_matches!(item, Ty::Array { .. });
    }

    #[test]
    fn parse_slice() {
        // deprecated syntax
        let item = parse::<Ty>("__slice[T]");
        assert_matches!(item, Ty::Slice { .. });

        // " new"  syntax
        let item = parse::<Ty>("[T]");
        assert_matches!(item, Ty::Slice { .. });

        let item = parse::<Ty>("&[T]");
        assert_matches!(item, Ty::Ref { ty, .. } if matches!(&*ty, Ty::Slice { .. }));
    }

    #[test]
    fn parse_ref() {
        let item = parse::<Ty>(
            r#"
            &T
            "#,
        );
        assert_matches!(
            item,
            Ty::Ref {
                mut_token: None,
                ..
            }
        );
    }

    #[test]
    fn parse_mut_ref() {
        let item = parse::<Ty>(
            r#"
            &mut T
            "#,
        );
        assert_matches!(
            item,
            Ty::Ref {
                mut_token: Some(_),
                ..
            }
        );
    }
}
