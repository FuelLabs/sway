use std::collections::HashMap;

use generational_arena::Index;
use sway_types::{Ident, Span};

use crate::error::{err, ok};
use crate::semantic_analysis::TypedExpression;
use crate::{CompileError, MatchCondition, TypeInfo};
use crate::{CompileResult, Literal, Scrutinee};

pub(crate) fn check_match_expression_exhaustivity(
    variable_created: Ident,
    cases_covered: Vec<MatchCondition>,
    span: Span,
    namespace: Index,
) -> CompileResult<()> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let exp = check!(
        TypedExpression::type_check_variable_expression(
            variable_created.clone(),
            variable_created.span().clone(),
            namespace
        ),
        return err(warnings, errors),
        warnings,
        errors
    );
    // collect all non-catchall scrutinees
    let mut scrutinees = vec![];
    for case in cases_covered.into_iter() {
        match case {
            MatchCondition::CatchAll(_) => {
                // catchall always creates exhaustivity
                return ok((), vec![], vec![]);
            }
            MatchCondition::Scrutinee(scrutinee) => {
                scrutinees.push(scrutinee);
            }
        }
    }
    check_match_expression_exhaustivity_inner(exp.return_type, scrutinees, span)
}

pub(crate) fn check_match_expression_exhaustivity_inner(
    type_id: usize,
    scrutinees: Vec<Scrutinee>,
    span: Span,
) -> CompileResult<()> {
    for scrutinee in scrutinees.iter() {
        if let Scrutinee::Variable { .. } = scrutinee {
            // variable scrutinee always creates exhaustivity
            return ok((), vec![], vec![]);
        }
    }
    let type_info = crate::type_engine::look_up_type_id(type_id);
    match type_info {
        TypeInfo::UnsignedInteger(_) => check_exhaustivity_unsigned_integer(scrutinees, span),
        TypeInfo::Boolean => check_exhaustivity_boolean(scrutinees, span),
        TypeInfo::Tuple(tuple_elem_types) => {
            check_exhaustivity_tuple(tuple_elem_types, scrutinees, span)
        }
        _ => unimplemented!(),
    }
}

fn check_exhaustivity_unsigned_integer(
    _scrutinees: Vec<Scrutinee>,
    span: Span,
) -> CompileResult<()> {
    // TODO: Theoretically it should be possible for someone to write a
    // match expression that enumerates all unsigned integers as match
    // arms. However, it is *really* unlikely that someone will do that.
    // So, this is left as a TODO.
    let errors = vec![CompileError::MatchExpressionNonExhaustive { span }];
    err(vec![], errors)
}

fn check_exhaustivity_boolean(scrutinees: Vec<Scrutinee>, span: Span) -> CompileResult<()> {
    let mut errors = vec![];
    let mut true_flag = false;
    let mut false_flag = false;
    for scrutinee in scrutinees.into_iter() {
        if let Scrutinee::Literal {
            value: Literal::Boolean(the_bool),
            ..
        } = scrutinee
        {
            if the_bool {
                true_flag = true;
            } else {
                false_flag = true;
            }
        }
    }
    if true_flag && false_flag {
        ok((), vec![], errors)
    } else {
        errors.push(CompileError::MatchExpressionNonExhaustive { span });
        err(vec![], errors)
    }
}

#[allow(clippy::map_entry)]
fn check_exhaustivity_tuple(
    tuple_elem_types: Vec<usize>,
    scrutinees: Vec<Scrutinee>,
    span: Span,
) -> CompileResult<()> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut vertically_gathered_scrutinees: HashMap<usize, Vec<Scrutinee>> = HashMap::new();
    for scrutinee in scrutinees.into_iter() {
        if let Scrutinee::Tuple { elems, span: _ } = scrutinee {
            for (pos, elem) in elems.into_iter().enumerate() {
                if vertically_gathered_scrutinees.contains_key(&pos) {
                    vertically_gathered_scrutinees
                        .get_mut(&pos)
                        .unwrap()
                        .push(elem);
                } else {
                    vertically_gathered_scrutinees.insert(pos, vec![elem]);
                }
            }
        }
    }
    for (pos, tuple_elem_type) in tuple_elem_types.into_iter().enumerate() {
        let new_scrutinees = vertically_gathered_scrutinees.get(&pos).unwrap().to_owned();
        check!(
            check_match_expression_exhaustivity_inner(
                tuple_elem_type,
                new_scrutinees,
                span.clone()
            ),
            return err(warnings, errors),
            warnings,
            errors
        );
    }
    ok((), warnings, errors)
}
