use crate::{
    build_config::BuildConfig,
    error::{err, ok, CompileError, CompileResult, ParserLifter},
    error_recovery_exp, parse_array_index,
    parse_tree::{ident, Expression},
    parser::Rule,
};

use sway_types::{span::Span, Ident};

use pest::iterators::Pair;

/// Represents the left hand side of a reassignment, which could either be a regular variable
/// expression, denoted by [ReassignmentTarget::VariableExpression], or, a storage field, denoted
/// by [ReassignmentTarget::StorageField].
#[derive(Debug, Clone)]
pub enum ReassignmentTarget {
    VariableExpression(Box<Expression>),
    StorageField(Vec<Ident>),
}

#[derive(Debug, Clone)]
pub struct Reassignment {
    // the thing being reassigned
    pub lhs: ReassignmentTarget,
    // the expression that is being assigned to the lhs
    pub rhs: Expression,
    pub(crate) span: Span,
}

impl Reassignment {
    pub fn lhs_span(&self) -> Span {
        match &self.lhs {
            ReassignmentTarget::VariableExpression(var) => match **var {
                Expression::SubfieldExpression { ref span, .. } => span.clone(),
                Expression::VariableExpression { ref name, .. } => name.span().clone(),
                _ => {
                    unreachable!("any other reassignment lhs is invalid and cannot be constructed.")
                }
            },
            ReassignmentTarget::StorageField(ref idents) => {
                idents.iter().fold(idents[0].span().clone(), |acc, ident| {
                    Span::join(acc, ident.span().clone())
                })
            }
        }
    }
    pub(crate) fn parse_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<ParserLifter<Reassignment>> {
        let path = config.map(|c| c.path());
        let span = Span::from_pest(pair.as_span(), path.clone());
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut iter = pair.into_inner();
        let variable_or_struct_reassignment = iter.next().expect("guaranteed by grammar");
        match variable_or_struct_reassignment.as_rule() {
            Rule::variable_reassignment => {
                let (name_result, mut body_result) = check!(
                    parse_simple_variable_reassignment(
                        variable_or_struct_reassignment,
                        config,
                        path,
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                let mut var_decls = name_result.var_decls;
                var_decls.append(&mut body_result.var_decls);
                let reassign = Reassignment {
                    lhs: ReassignmentTarget::VariableExpression(Box::new(name_result.value)),
                    rhs: body_result.value,
                    span,
                };

                ok(
                    ParserLifter {
                        var_decls,
                        value: reassign,
                    },
                    warnings,
                    errors,
                )
            }
            Rule::struct_field_reassignment => {
                let (mut expr_result, body_result) = check!(
                    parse_subfield_reassignment(variable_or_struct_reassignment, config, path),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                let mut var_decls = body_result.var_decls;
                var_decls.append(&mut expr_result.var_decls);
                let exp = Reassignment {
                    lhs: ReassignmentTarget::VariableExpression(Box::new(expr_result.value)),
                    rhs: body_result.value,
                    span,
                };

                ok(
                    ParserLifter {
                        var_decls,
                        value: exp,
                    },
                    warnings,
                    errors,
                )
            }
            Rule::storage_reassignment => {
                // there is some ugly handling of the pest data structure here
                // because we are reusing code from struct field reassignments
                // this leads to some nested `Rule`s, so the actual storage
                // reassignment requires some `expect()`s to extract.
                let mut parts = variable_or_struct_reassignment.into_inner();
                let _storage_keyword = parts.next();
                let mut parts = parts.collect::<Vec<_>>();
                let rhs = parts.pop();
                assert_eq!(rhs.as_ref().map(|x| x.as_rule()), Some(Rule::expr));
                let rhs = rhs.expect("guaranteed by grammar");
                let rhs = check!(
                    Expression::parse_from_pair(rhs, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let mut lhs = Vec::new();
                for item in parts {
                    lhs.push(check!(
                        ident::parse_from_pair(item, config),
                        continue,
                        warnings,
                        errors
                    ))
                }

                let reassign = Reassignment {
                    lhs: ReassignmentTarget::StorageField(lhs),
                    rhs: rhs.value,
                    span,
                };
                ok(
                    ParserLifter {
                        var_decls: rhs.var_decls,
                        value: reassign,
                    },
                    warnings,
                    errors,
                )
            }
            _ => unreachable!("guaranteed by grammar"),
        }
    }
}

fn parse_simple_variable_reassignment(
    pair: Pair<Rule>,
    config: Option<&BuildConfig>,
    path: Option<std::sync::Arc<std::path::PathBuf>>,
) -> CompileResult<(ParserLifter<Expression>, ParserLifter<Expression>)> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut iter = pair.into_inner();
    let name_result = check!(
        Expression::parse_from_pair(iter.next().unwrap(), config),
        return err(warnings, errors),
        warnings,
        errors
    );
    let body = iter.next().unwrap();
    let body_result = check!(
        Expression::parse_from_pair(body.clone(), config),
        ParserLifter::empty(error_recovery_exp(Span::from_pest(body.as_span(), path))),
        warnings,
        errors
    );

    ok((name_result, body_result), warnings, errors)
}

fn parse_subfield_reassignment(
    pair: Pair<Rule>,
    config: Option<&BuildConfig>,
    path: Option<std::sync::Arc<std::path::PathBuf>>,
) -> CompileResult<(ParserLifter<Expression>, ParserLifter<Expression>)> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut iter = pair.into_inner();
    let lhs = iter.next().expect("guaranteed by grammar");
    let rhs = iter.next().expect("guaranteed by grammar");
    let rhs_span = Span::from_pest(rhs.as_span(), path.clone());
    let body_result = check!(
        Expression::parse_from_pair(rhs, config),
        ParserLifter::empty(error_recovery_exp(rhs_span)),
        warnings,
        errors
    );
    let inner = lhs.into_inner().next().expect("guaranteed by grammar");
    assert_eq!(inner.as_rule(), Rule::subfield_path);
    // treat parent as one expr, final name as the field to be accessed
    // if there are multiple fields, this is a nested expression
    // i.e. `a.b.c` is a lookup of field `c` on `a.b` which is a lookup
    // of field `b` on `a`
    // the first thing is either an exp or a var, everything subsequent must be
    // a field
    let mut name_parts = inner.into_inner();
    let mut expr_result = check!(
        parse_subfield_path_ensure_only_var(
            name_parts.next().expect("guaranteed by grammar"),
            config
        ),
        return err(warnings, errors),
        warnings,
        errors
    );

    for name_part in name_parts {
        let expr = Expression::SubfieldExpression {
            prefix: Box::new(expr_result.value.clone()),
            span: Span::from_pest(name_part.as_span(), path.clone()),
            field_to_access: check!(
                ident::parse_from_pair(name_part, config),
                continue,
                warnings,
                errors
            ),
        };
        expr_result = ParserLifter {
            var_decls: expr_result.var_decls,
            value: expr,
        };
    }

    ok((expr_result, body_result), warnings, errors)
}

fn parse_subfield_path_ensure_only_var(
    item: Pair<Rule>,
    config: Option<&BuildConfig>,
) -> CompileResult<ParserLifter<Expression>> {
    let warnings = vec![];
    let mut errors = vec![];
    let path = config.map(|c| c.path());
    let item = item.into_inner().next().expect("guarenteed by grammar");
    match item.as_rule() {
        Rule::call_item => parse_call_item_ensure_only_var(item, config),
        Rule::array_index => parse_array_index(item, config),
        a => {
            eprintln!(
                "Unimplemented subfield path: {:?} ({:?}) ({:?})",
                a,
                item.as_str(),
                item.as_rule()
            );
            errors.push(CompileError::UnimplementedRule(
                a,
                Span::from_pest(item.as_span(), path.clone()),
            ));
            // construct unit expression for error recovery
            let exp_result =
                ParserLifter::empty(error_recovery_exp(Span::from_pest(item.as_span(), path)));
            ok(exp_result, warnings, errors)
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
fn parse_call_item_ensure_only_var(
    item: Pair<Rule>,
    config: Option<&BuildConfig>,
) -> CompileResult<ParserLifter<Expression>> {
    let path = config.map(|c| c.path());
    let mut warnings = vec![];
    let mut errors = vec![];
    assert_eq!(item.as_rule(), Rule::call_item);
    let item = item.into_inner().next().expect("guaranteed by grammar");
    let exp = match item.as_rule() {
        Rule::ident => Expression::VariableExpression {
            name: check!(
                ident::parse_from_pair(item.clone(), config),
                return err(warnings, errors),
                warnings,
                errors
            ),
            span: Span::from_pest(item.as_span(), path),
        },
        Rule::expr => {
            errors.push(CompileError::InvalidExpressionOnLhs {
                span: Span::from_pest(item.as_span(), path),
            });
            return err(warnings, errors);
        }
        a => unreachable!("{:?}", a),
    };
    ok(
        ParserLifter {
            var_decls: vec![],
            value: exp,
        },
        warnings,
        errors,
    )
}
