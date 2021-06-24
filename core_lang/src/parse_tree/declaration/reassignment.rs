use crate::error::{err, ok, CompileError, CompileResult};
use crate::parse_tree::Expression;
use crate::parser::Rule;
use crate::Ident;
use pest::iterators::Pair;
use pest::Span;

#[derive(Debug, Clone)]
pub struct Reassignment<'sc> {
    // the thing being reassigned
    pub lhs: Box<Expression<'sc>>,
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
                let name = eval!(
                    Expression::parse_from_pair_inner,
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
                        lhs: Box::new(name),
                        rhs: body,
                        span,
                    },
                    warnings,
                    errors,
                )
            }
            Rule::struct_field_reassignment => {
                let mut iter = variable_or_struct_reassignment.into_inner();
                let lhs = iter.next().expect("guaranteed by grammar");
                let rhs = iter.next().expect("guaranteed by grammar");
                let rhs_span = rhs.as_span();
                let body = eval!(
                    Expression::parse_from_pair,
                    warnings,
                    errors,
                    rhs,
                    Expression::Unit { span: rhs_span }
                );

                let inner = lhs.into_inner().next().expect("guaranteed by gramar");
                assert_eq!(inner.as_rule(), Rule::subfield_path);
                let name_parts = inner.into_inner().collect::<Vec<_>>();

                // treat parent as one expr, final name as the field to be accessed
                // if there are multiple fields, this is a nested expression
                // i.e. `a.b.c` is a lookup of field `c` on `a.b` which is a lookup
                // of field `b` on `a`
                // the first thing is either an exp or a var, everything subsequent must be
                // a field
                let mut name_parts = name_parts.into_iter();
                let mut expr = eval!(
                    parse_call_item_ensure_only_var,
                    warnings,
                    errors,
                    name_parts.next().expect("guaranteed by grammar"),
                    return err(warnings, errors)
                );

                for name_part in name_parts {
                    expr = Expression::SubfieldExpression {
                        prefix: Box::new(expr.clone()),
                        unary_op: None, // TODO
                        span: name_part.as_span(),
                        field_to_access: eval!(
                            Ident::parse_from_pair,
                            warnings,
                            errors,
                            name_part,
                            continue
                        ),
                    }
                }

                ok(
                    Reassignment {
                        lhs: Box::new(expr),
                        rhs: body,
                        span,
                    },
                    warnings,
                    errors,
                )
            }
            _ => unreachable!("guaranteed by grammar"),
        }
    }
}
/// Parses a `call_item` rule but ensures that it is only a variable expression, since generic
/// expressions on the LHS of a reassignment are invalid.
/// valid:
/// ```ignore
/// x.y.foo = 5;
/// ```
///
/// invalid:
/// ```ignore
/// (foo()).x = 5;
/// ```
fn parse_call_item_ensure_only_var<'sc>(
    item: Pair<'sc, Rule>,
) -> CompileResult<'sc, Expression<'sc>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    assert_eq!(item.as_rule(), Rule::call_item);
    let item = item.into_inner().next().expect("guaranteed by grammar");
    let exp = match item.as_rule() {
        Rule::ident => Expression::VariableExpression {
            name: eval!(
                Ident::parse_from_pair,
                warnings,
                errors,
                item,
                return err(warnings, errors)
            ),
            span: item.as_span(),
            unary_op: None,
        },
        Rule::expr => {
            errors.push(CompileError::InvalidExpressionOnLhs {
                span: item.as_span(),
            });
            return err(warnings, errors);
        }
        a => unreachable!("{:?}", a),
    };
    ok(exp, warnings, errors)
}
