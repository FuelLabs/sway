use crate::{Parse, ParseBracket, ParseResult, Parser, Peek};

use sway_ast::brackets::{Braces, Parens};
use sway_ast::keywords::{DoubleDotToken, FalseToken, TrueToken};
use sway_ast::literal::{LitBool, LitBoolType};
use sway_ast::punctuated::Punctuated;
use sway_ast::{Literal, PathExpr, Pattern, PatternStructField};
use sway_error::parser_error::ParseErrorKind;
use sway_types::Spanned;

impl Parse for Pattern {
    fn parse(parser: &mut Parser) -> ParseResult<Pattern> {
        let combine = |lhs, rhs, pipe_token| Pattern::Or {
            lhs,
            pipe_token,
            rhs,
        };
        parse_binary(parser, parse_atomic_pattern, combine)
    }
}

fn parse_binary<O: Peek>(
    parser: &mut Parser,
    sub: impl Fn(&mut Parser) -> ParseResult<Pattern>,
    combine: impl Fn(Box<Pattern>, Box<Pattern>, O) -> Pattern,
) -> ParseResult<Pattern> {
    let mut expr = sub(parser)?;
    while let Some((op_token, rhs)) = parse_op_rhs(parser, &sub)? {
        expr = combine(Box::new(expr), rhs, op_token);
    }
    Ok(expr)
}

fn parse_op_rhs<O: Peek>(
    parser: &mut Parser,
    sub: impl Fn(&mut Parser) -> ParseResult<Pattern>,
) -> ParseResult<Option<(O, Box<Pattern>)>> {
    if let Some(op_token) = parser.take() {
        let rhs = Box::new(sub(parser)?);
        return Ok(Some((op_token, rhs)));
    }
    Ok(None)
}

fn parse_atomic_pattern(parser: &mut Parser) -> ParseResult<Pattern> {
    {
        let ref_token = parser.take();
        let mut_token = parser.take();
        if ref_token.is_some() || mut_token.is_some() {
            let name = parser.parse()?;
            return Ok(Pattern::Var {
                reference: ref_token,
                mutable: mut_token,
                name,
            });
        }

        let lit_bool = |span, kind| Ok(Pattern::Literal(Literal::Bool(LitBool { span, kind })));

        if let Some(ident) = parser.take::<TrueToken>() {
            return lit_bool(ident.span(), LitBoolType::True);
        }
        if let Some(ident) = parser.take::<FalseToken>() {
            return lit_bool(ident.span(), LitBoolType::False);
        }
        if let Some(literal) = parser.take() {
            return Ok(Pattern::Literal(literal));
        }
        if let Some(tuple) = Parens::try_parse(parser)? {
            return Ok(Pattern::Tuple(tuple));
        }
        if let Some(underscore_token) = parser.take() {
            return Ok(Pattern::Wildcard { underscore_token });
        }

        let path = parser.parse::<PathExpr>()?;
        if path.incomplete_suffix {
            return Ok(Pattern::Error(
                Box::new([path.span()]),
                parser.emit_error(ParseErrorKind::ExpectedPathType),
            ));
        }
        if let Some(args) = Parens::try_parse(parser)? {
            return Ok(Pattern::Constructor { path, args });
        }
        if let Some(fields) = Braces::try_parse(parser)? {
            let inner_fields: &Punctuated<_, _> = fields.get();
            let rest_pattern = inner_fields
                .value_separator_pairs
                .iter()
                .find(|(p, _)| matches!(p, PatternStructField::Rest { token: _ }));

            if let Some((rest_pattern, _)) = rest_pattern {
                return Err(parser.emit_error_with_span(
                    ParseErrorKind::UnexpectedRestPattern,
                    rest_pattern.span(),
                ));
            }

            return Ok(Pattern::Struct { path, fields });
        }
        match path.try_into_ident() {
            Ok(name) => Ok(Pattern::AmbiguousSingleIdent(name)),
            Err(path) => Ok(Pattern::Constant(path)),
        }
    }
}

impl Parse for PatternStructField {
    fn parse(parser: &mut Parser) -> ParseResult<PatternStructField> {
        if let Some(token) = parser.take::<DoubleDotToken>() {
            return Ok(PatternStructField::Rest { token });
        }

        let field_name = parser.parse()?;
        let pattern_opt = match parser.take() {
            Some(colon_token) => Some((colon_token, parser.parse()?)),
            None => None,
        };
        Ok(PatternStructField::Field {
            field_name,
            pattern_opt,
        })
    }
}
