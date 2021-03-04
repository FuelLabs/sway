use crate::error::{err, ok, CompileResult};
use crate::parse_tree::{Expression, VarName};
use crate::parser::Rule;
use pest::iterators::Pair;
use pest::Span;

#[derive(Debug, Clone)]
pub(crate) struct Reassignment<'sc> {
    // the thing being reassigned
    pub(crate) lhs: VarName<'sc>,
    // the expression that is being assigned to the lhs
    pub(crate) rhs: Expression<'sc>,
    pub(crate) span: Span<'sc>,
}

impl<'sc> Reassignment<'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<Self> {
        let span = pair.as_span();
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut iter = pair.into_inner();
        let name = eval!(
            VarName::parse_from_pair,
            warnings,
            errors,
            iter.next().unwrap(),
            return err(warnings, errors)
        );
        let body = iter.next().unwrap();
        let body = eval!(
            Expression::parse_from_pair,
            warnings,
            errors,
            body.clone(),
            Expression::Unit {
                span: body.as_span()
            }
        );

        ok(
            Reassignment {
                lhs: name,
                rhs: body,
                span,
            },
            warnings,
            errors,
        )
    }
}
