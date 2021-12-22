use crate::build_config::BuildConfig;
use crate::error::*;
use crate::parser::Rule;
use crate::span;
use crate::CatchAll;
use crate::CodeBlock;
use pest::iterators::Pair;

use super::scrutinee::Scrutinee;
use super::{Expression, MatchCondition};

#[derive(Debug, Clone)]
pub struct MatchBranch {
    pub(crate) condition: MatchCondition,
    pub(crate) result: Expression,
    pub(crate) span: span::Span,
}

impl MatchBranch {
    pub fn parse_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Self> {
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
                        path,
                    },
                ));
                return err(warnings, errors);
            }
        };
        let condition = match condition.into_inner().next() {
            Some(e) => {
                match e.as_rule() {
                    Rule::catch_all => MatchCondition::CatchAll(CatchAll {
                        span: span::Span {
                            span: e.as_span(),
                            path: path.clone(),
                        },
                    }),
                    Rule::scrutinee => {
                        let scrutinee = check!(
                            Scrutinee::parse_from_pair(e, config),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        MatchCondition::Scrutinee(scrutinee)
                    }
                    a => {
                        eprintln!(
                            "Unimplemented condition: {:?} ({:?}) ({:?})",
                            a,
                            e.as_str(),
                            e.as_rule()
                        );
                        errors.push(CompileError::UnimplementedRule(
                            a,
                            span::Span {
                                span: e.as_span(),
                                path: path.clone(),
                            },
                        ));
                        // construct unit expression for error recovery
                        MatchCondition::CatchAll(CatchAll {
                            span: span::Span {
                                span: e.as_span(),
                                path: path.clone(),
                            },
                        })
                    }
                }
            }
            None => {
                errors.push(CompileError::Internal(
                    "Unexpected empty iterator in match condition parsing.",
                    span::Span {
                        span: pair.as_span(),
                        path,
                    },
                ));
                return err(warnings, errors);
            }
        };
        let result = match branch.next() {
            Some(o) => o,
            None => {
                errors.push(CompileError::Internal(
                    "Unexpected empty iterator in match branch parsing.",
                    span::Span {
                        span: pair.as_span(),
                        path,
                    },
                ));
                return err(warnings, errors);
            }
        };
        let result = match result.as_rule() {
            Rule::expr => check!(
                Expression::parse_from_pair(result.clone(), config),
                Expression::Tuple {
                    fields: vec![],
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
                    path,
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
