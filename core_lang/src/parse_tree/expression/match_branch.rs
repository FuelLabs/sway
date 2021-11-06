use crate::build_config::BuildConfig;
use crate::error::*;
use crate::parser::Rule;
use crate::span;
use crate::CodeBlock;
use pest::iterators::Pair;

use super::{Expression, MatchCondition};

#[derive(Debug, Clone)]
pub struct MatchBranch<'sc> {
    pub(crate) condition: MatchCondition<'sc>,
    pub(crate) result: Expression<'sc>,
    #[allow(dead_code)]
    // this span may be used for errors in the future, although it is not right now.
    pub(crate) span: span::Span<'sc>,
}

impl<'sc> MatchBranch<'sc> {
    pub fn parse_from_pair(
        pair: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        let path = config.map(|c| c.path());
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let span = span::Span {
            span: pair.as_span(),
            path: path.clone(),
        };
        let mut branch = pair.clone().into_inner();
        let condition = match branch.next() {
            Some(o) => o,
            None => {
                errors.push(CompileError::Internal(
                    "Unexpected empty iterator in match branch parsing.",
                    span::Span {
                        span: pair.as_span(),
                        path: path.clone(),
                    },
                ));
                return err(warnings, errors);
            }
        };
        let condition = match condition.into_inner().next() {
            Some(e) => {
                let expr = check!(
                    Expression::parse_from_pair(e.clone(), config),
                    Expression::Unit {
                        span: span::Span {
                            span: e.as_span(),
                            path: path.clone()
                        }
                    },
                    warnings,
                    errors
                );
                MatchCondition::Expression(expr)
            }
            // the "_" case
            None => MatchCondition::CatchAll,
        };
        let result = match branch.next() {
            Some(o) => o,
            None => {
                errors.push(CompileError::Internal(
                    "Unexpected empty iterator in match branch parsing.",
                    span::Span {
                        span: pair.as_span(),
                        path: path.clone(),
                    },
                ));
                return err(warnings, errors);
            }
        };
        let result = match result.as_rule() {
            Rule::expr => check!(
                Expression::parse_from_pair(result.clone(), config),
                Expression::Unit {
                    span: span::Span {
                        span: result.as_span(),
                        path
                    }
                },
                warnings,
                errors
            ),
            Rule::code_block => {
                let span = span::Span {
                    span: result.as_span(),
                    path: path.clone(),
                };
                Expression::CodeBlock {
                    contents: check!(
                        CodeBlock::parse_from_pair(result, config),
                        CodeBlock {
                            contents: Vec::new(),
                            whole_block_span: span.clone(),
                        },
                        warnings,
                        errors
                    ),
                    span,
                }
            }
            _ => unreachable!(),
        };
        ok(
            MatchBranch {
                condition,
                result,
                span,
            },
            warnings,
            errors,
        )
    }
}
