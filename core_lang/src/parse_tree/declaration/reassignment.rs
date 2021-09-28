use crate::build_config::BuildConfig;
use crate::error::{err, ok, CompileError, CompileResult};
use crate::parse_tree::Expression;
use crate::parser::Rule;
use crate::span::Span;
use crate::Ident;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct Reassignment<'sc> {
    // the thing being reassigned
    pub lhs: Box<Expression<'sc>>,
    // the expression that is being assigned to the lhs
    pub rhs: Expression<'sc>,
    pub(crate) span: Span<'sc>,
}

impl<'sc> Reassignment<'sc> {
    pub(crate) fn parse_from_pair(
        pair: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Reassignment<'sc>> {
        let path = config.map(|c| c.path());
        let span = Span {
            span: pair.as_span(),
            path: path.clone(),
        };
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut iter = pair.into_inner();
        let variable_or_struct_reassignment = iter.next().expect("guaranteed by grammar");
        match variable_or_struct_reassignment.as_rule() {
            Rule::variable_reassignment => {
                let mut iter = variable_or_struct_reassignment.into_inner();
                let name = check!(
                    Expression::parse_from_pair_inner(iter.next().unwrap(), config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let body = iter.next().unwrap();
                let body = check!(
                    Expression::parse_from_pair(body.clone(), config),
                    Expression::Unit {
                        span: Span {
                            span: body.as_span(),
                            path
                        }
                    },
                    warnings,
                    errors
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
                let rhs_span = Span {
                    span: rhs.as_span(),
                    path: path.clone(),
                };
                let body = check!(
                    Expression::parse_from_pair(rhs, config),
                    Expression::Unit { span: rhs_span },
                    warnings,
                    errors
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
                let mut expr = check!(
                    parse_call_item_ensure_only_var(
                        name_parts.next().expect("guaranteed by grammar"),
                        config
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                for name_part in name_parts {
                    expr = Expression::SubfieldExpression {
                        prefix: Box::new(expr.clone()),
                        span: Span {
                            span: name_part.as_span(),
                            path: path.clone(),
                        },
                        field_to_access: check!(
                            Ident::parse_from_pair(name_part, config),
                            continue,
                            warnings,
                            errors
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
    config: Option<&BuildConfig>,
) -> CompileResult<'sc, Expression<'sc>> {
    let path = config.map(|c| c.path());
    let mut warnings = vec![];
    let mut errors = vec![];
    assert_eq!(item.as_rule(), Rule::call_item);
    let item = item.into_inner().next().expect("guaranteed by grammar");
    let exp = match item.as_rule() {
        Rule::ident => Expression::VariableExpression {
            name: check!(
                Ident::parse_from_pair(item.clone(), config),
                return err(warnings, errors),
                warnings,
                errors
            ),
            span: Span {
                span: item.as_span(),
                path: path.clone(),
            },
        },
        Rule::expr => {
            errors.push(CompileError::InvalidExpressionOnLhs {
                span: Span {
                    span: item.as_span(),
                    path: path.clone(),
                },
            });
            return err(warnings, errors);
        }
        a => unreachable!("{:?}", a),
    };
    ok(exp, warnings, errors)
}
