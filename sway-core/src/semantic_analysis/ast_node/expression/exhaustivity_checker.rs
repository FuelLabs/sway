use generational_arena::Index;
use sway_types::{Ident, Span};

use crate::error::{err, ok};
use crate::semantic_analysis::TypedExpression;
use crate::CompileResult;
use crate::{CompileError, MatchCondition, TypeInfo};

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
    let type_info = crate::type_engine::look_up_type_id(exp.return_type);
    match type_info {
        TypeInfo::UnsignedInteger(_) => check_exhaustivity_unsigned_integer(cases_covered, span),
        _ => unimplemented!(),
    }
}

fn check_exhaustivity_unsigned_integer(
    cases_covered: Vec<MatchCondition>,
    span: Span,
) -> CompileResult<()> {
    let mut errors = vec![];
    let mut catchall_flag = false;
    for case in cases_covered.into_iter() {
        if let MatchCondition::CatchAll(_) = case {
            catchall_flag = true;
        }
    }
    if catchall_flag {
        ok((), vec![], errors)
    } else {
        errors.push(CompileError::MatchExpressionNonExhaustive { span });
        err(vec![], errors)
    }
}
