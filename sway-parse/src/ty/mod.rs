use crate::{Parse, ParseBracket, ParseResult, ParseToEnd, Parser, ParserConsumed};

use sway_ast::brackets::{Parens, SquareBrackets};
use sway_ast::keywords::{DoubleColonToken, OpenAngleBracketToken};
use sway_ast::ty::{Ty, TyArrayDescriptor, TyTupleDescriptor};
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
        if let Some(descriptor) = SquareBrackets::try_parse(parser)? {
            return Ok(Ty::Array(descriptor));
        };

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
        if let Some(ptr_token) = parser.take() {
            let ty = SquareBrackets::parse_all_inner(parser, |mut parser| {
                parser.emit_error(ParseErrorKind::UnexpectedTokenAfterPtrType)
            })?;
            return Ok(Ty::Ptr { ptr_token, ty });
        }
        if let Some(slice_token) = parser.take() {
            let ty = SquareBrackets::parse_all_inner(parser, |mut parser| {
                parser.emit_error(ParseErrorKind::UnexpectedTokenAfterSliceType)
            })?;
            return Ok(Ty::Slice { slice_token, ty });
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
    fn parse_slice() {
        let item = parse::<Ty>(
            r#"
            __slice[T]
            "#,
        );
        assert_matches!(item, Ty::Slice { .. });
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
