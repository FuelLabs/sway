use crate::error::{err, ok, CompileResult};
use crate::parse_tree::Expression;
use crate::parser::Rule;
use crate::Ident;
use pest::iterators::Pair;
use pest::Span;

#[derive(Debug, Clone)]
pub struct Reassignment<'sc> {
    // the thing being reassigned
    pub lhs: Ident<'sc>,
    // the expression that is being assigned to the lhs
    pub rhs: Expression<'sc>,
    pub(crate) span: Span<'sc>,
}

impl<'sc> Reassignment<'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<Self> {
        let span = pair.as_span();
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut iter = pair.into_inner();
        let variable_or_struct_reassignment = iter.next().expect("guaranteed by grammar");
        match variable_or_struct_reassignment.as_rule() {
            Rule::variable_reassignment => {
                let mut iter = variable_or_struct_reassignment.into_inner();
                dbg!(&iter);
                let name = eval!(
                    Ident::parse_from_pair,
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
            Rule::struct_field_reassignment => todo!("parse the struct name and the field to be reassigned -- recursive definition since the field could be nested. "),
            _ => unreachable!("guaranteed by grammar"),
        }
    }
}
