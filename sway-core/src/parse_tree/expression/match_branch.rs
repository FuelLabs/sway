use crate::{build_config::BuildConfig, error::*, parser::Rule, CodeBlock};

use sway_types::Span;

use pest::iterators::Pair;

use super::{scrutinee::Scrutinee, Expression};

#[derive(Debug, Clone)]
pub struct MatchBranch {
    pub scrutinee: Scrutinee,
    pub result: Expression,
    pub(crate) span: Span,
}

impl MatchBranch {
    pub fn parse_from_pair(pair: Pair<Rule>, config: Option<&BuildConfig>) -> CompileResult<Self> {
        let path = config.map(|c| c.path());
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let span = Span::from_pest(pair.as_span(), path.clone());
        let mut branch = pair.clone().into_inner();
        let scrutinee = match branch.next() {
            Some(o) => o,
            None => {
                errors.push(CompileError::Internal(
                    "Unexpected empty iterator in match branch parsing.",
                    Span::from_pest(pair.as_span(), path),
                ));
                return err(warnings, errors);
            }
        };
        let scrutinee = match scrutinee.into_inner().next() {
            Some(e) => {
                check!(
                    Scrutinee::parse_from_pair(e, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            None => {
                errors.push(CompileError::Internal(
                    "Unexpected empty iterator in match condition parsing.",
                    Span::from_pest(pair.as_span(), path),
                ));
                return err(warnings, errors);
            }
        };
        let result = match branch.next() {
            Some(o) => o,
            None => {
                errors.push(CompileError::Internal(
                    "Unexpected empty iterator in match branch parsing.",
                    Span::from_pest(pair.as_span(), path),
                ));
                return err(warnings, errors);
            }
        };
        let result = match result.as_rule() {
            Rule::code_block => {
                let span = Span::from_pest(result.as_span(), path);
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
                scrutinee,
                result,
                span,
            },
            warnings,
            errors,
        )
    }
}
