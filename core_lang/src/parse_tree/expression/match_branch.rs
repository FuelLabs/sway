use crate::error::*;
use crate::parser::Rule;
use crate::CodeBlock;
use pest::iterators::Pair;
use pest::Span;
use std::collections::HashMap;

use super::{Expression, MatchCondition};

#[derive(Debug, Clone)]
pub struct MatchBranch<'sc> {
    pub(crate) condition: MatchCondition<'sc>,
    pub(crate) result: Expression<'sc>,
    pub(crate) span: Span<'sc>,
}

impl<'sc> MatchBranch<'sc> {
    pub fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let span = pair.as_span();
        let mut branch = pair.clone().into_inner();
        let condition = match branch.next() {
            Some(o) => o,
            None => {
                errors.push(CompileError::Internal(
                    "Unexpected empty iterator in match branch parsing.",
                    pair.as_span(),
                ));
                return err(warnings, errors);
            }
        };
        let condition = match condition.into_inner().next() {
            Some(e) => {
                let expr = eval!(
                    Expression::parse_from_pair,
                    warnings,
                    errors,
                    e,
                    Expression::Unit { span: e.as_span() }
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
                    pair.as_span(),
                ));
                return err(warnings, errors);
            }
        };
        let result = match result.as_rule() {
            Rule::expr => eval!(
                Expression::parse_from_pair,
                warnings,
                errors,
                result,
                Expression::Unit {
                    span: result.as_span()
                }
            ),
            Rule::code_block => {
                let span = result.as_span();
                Expression::CodeBlock {
                    contents: eval!(
                        CodeBlock::parse_from_pair,
                        warnings,
                        errors,
                        result,
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