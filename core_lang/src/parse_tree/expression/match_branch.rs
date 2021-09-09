use crate::error::*;
use crate::parser::Rule;
use crate::CodeBlock;
use pest::iterators::Pair;
use crate::span;
use crate::build_config::BuildConfig;
use std::collections::HashMap;

use super::{Expression, MatchCondition};

#[derive(Debug, Clone)]
pub struct MatchBranch<'sc> {
    pub(crate) condition: MatchCondition<'sc>,
    pub(crate) result: Expression<'sc>,
    pub(crate) span: span::Span<'sc>,
}

impl<'sc> MatchBranch<'sc> {
    pub fn parse_from_pair(
        pair: Pair<'sc, Rule>,
        config: Option<BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        let path = config.clone().map(|c| c.dir_of_code);
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
                let expr = eval2!(
                    Expression::parse_from_pair,
                    warnings,
                    errors,
                    e,
                    config.clone(),
                    Expression::Unit {
                        span: span::Span {
                            span: e.as_span(),
                            path: path.clone()
                        }
                    }
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
            Rule::expr => eval2!(
                Expression::parse_from_pair,
                warnings,
                errors,
                result,
                config.clone(),
                Expression::Unit {
                    span: span::Span {
                        span: result.as_span(),
                        path
                    }
                }
            ),
            Rule::code_block => {
                let span = span::Span {
                    span: result.as_span(),
                    path: path.clone(),
                };
                Expression::CodeBlock {
                    contents: eval2!(
                        CodeBlock::parse_from_pair,
                        warnings,
                        errors,
                        result,
                        config.clone(),
                        CodeBlock {
                            contents: Vec::new(),
                            whole_block_span: span.clone(),
                            scope: HashMap::default()
                        }
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