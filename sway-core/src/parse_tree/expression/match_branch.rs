use crate::{build_config::BuildConfig, error::*, parser::Rule, CatchAll, CodeBlock};

use sway_types::{span, Span};

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
    pub fn parse_from_pair(pair: Pair<Rule>, config: Option<&BuildConfig>) -> CompileResult<Self> {
        let path = config.map(|c| c.path());
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let span = span::Span::from_pest(pair.as_span(), path.clone());
        let mut branch = pair.clone().into_inner();
        let condition = match branch.next() {
            Some(o) => o,
            None => {
                errors.push(CompileError::Internal(
                    "Unexpected empty iterator in match branch parsing.",
                    span::Span::from_pest(pair.as_span(), path),
                ));
                return err(warnings, errors);
            }
        };
        let condition = match condition.into_inner().next() {
            Some(e) => {
                match e.as_rule() {
                    Rule::catch_all => MatchCondition::CatchAll(CatchAll {
                        span: span::Span::from_pest(e.as_span(), path.clone()),
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
                            span::Span::from_pest(e.as_span(), path.clone()),
                        ));
                        // construct unit expression for error recovery
                        MatchCondition::CatchAll(CatchAll {
                            span: span::Span::from_pest(e.as_span(), path.clone()),
                        })
                    }
                }
            }
            None => {
                errors.push(CompileError::Internal(
                    "Unexpected empty iterator in match condition parsing.",
                    span::Span::from_pest(pair.as_span(), path),
                ));
                return err(warnings, errors);
            }
        };
        let result = match branch.next() {
            Some(o) => o,
            None => {
                errors.push(CompileError::Internal(
                    "Unexpected empty iterator in match branch parsing.",
                    span::Span::from_pest(pair.as_span(), path),
                ));
                return err(warnings, errors);
            }
        };
        let result = match result.as_rule() {
            Rule::code_block => {
                let span = span::Span::from_pest(result.as_span(), path);
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
            a => {
                let span = Span::from_pest(result.as_span(), path);
                errors.push(CompileError::UnimplementedRule(a, span.clone()));
                Expression::Tuple {
                    fields: vec![],
                    span,
                }
            }
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
