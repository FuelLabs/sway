use sway_types::Span;

use crate::{
    error::{err, ok},
    language::{parsed::*, *},
    semantic_analysis::{
        ast_node::expression::typed_expression::{
            instantiate_if_expression, instantiate_lazy_operator,
        },
        IsConstant, TyExpression, TyExpressionVariant, TypeCheckContext,
    },
    type_system::{insert_type, TypeId},
    CompileError, CompileResult, TypeInfo,
};

use super::{typed_match_branch::TyMatchBranch, typed_scrutinee::TyScrutinee};

#[derive(Debug)]
pub(crate) struct TyMatchExpression {
    branches: Vec<TyMatchBranch>,
    return_type_id: TypeId,
    #[allow(dead_code)]
    span: Span,
}

impl TyMatchExpression {
    pub(crate) fn type_check(
        ctx: TypeCheckContext,
        typed_value: TyExpression,
        branches: Vec<MatchBranch>,
        span: Span,
    ) -> CompileResult<(TyMatchExpression, Vec<TyScrutinee>)> {
        let mut warnings = vec![];
        let mut errors = vec![];

        // type check all of the branches
        let mut typed_branches = vec![];
        let mut typed_scrutinees = vec![];
        let mut ctx =
            ctx.with_help_text("all branches of a match statement must return the same type");
        for branch in branches.into_iter() {
            let (typed_branch, typed_scrutinee) = check!(
                TyMatchBranch::type_check(ctx.by_ref(), &typed_value, branch),
                continue,
                warnings,
                errors
            );
            typed_branches.push(typed_branch);
            typed_scrutinees.push(typed_scrutinee);
        }

        if !errors.is_empty() {
            return err(warnings, errors);
        }

        let typed_exp = TyMatchExpression {
            branches: typed_branches,
            return_type_id: ctx.type_annotation(),
            span,
        };
        ok((typed_exp, typed_scrutinees), warnings, errors)
    }

    pub(crate) fn convert_to_typed_if_expression(
        self,
        mut ctx: TypeCheckContext,
    ) -> CompileResult<TyExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let TyMatchExpression { branches, .. } = self;

        // create the typed if expression object that we will be building on to
        let mut typed_if_exp: Option<TyExpression> = None;

        // for every branch of the match expression, in reverse
        for TyMatchBranch {
            conditions, result, ..
        } in branches.into_iter().rev()
        {
            // create the conditional that will act as the conditional for the if statement, in reverse
            let mut conditional: Option<TyExpression> = None;
            for (left_req, right_req) in conditions.into_iter().rev() {
                let joined_span = Span::join(left_req.span.clone(), right_req.span.clone());
                let args = vec![left_req, right_req];
                let new_condition = check!(
                    TyExpression::core_ops_eq(ctx.by_ref(), args, joined_span),
                    continue,
                    warnings,
                    errors
                );
                conditional = Some(match conditional {
                    Some(inner_condition) => {
                        let joined_span =
                            Span::join(inner_condition.span.clone(), new_condition.span.clone());
                        instantiate_lazy_operator(
                            LazyOp::And,
                            new_condition,
                            inner_condition,
                            insert_type(TypeInfo::Boolean),
                            joined_span,
                        )
                    }
                    None => new_condition,
                });
            }

            // add to the if expression that we are building using the result component
            // of the match branch and using the conditional that we just built
            let result_span = result.span.clone();
            typed_if_exp = Some(match (typed_if_exp.clone(), conditional) {
                (None, None) => result,
                (None, Some(conditional)) => {
                    check!(
                        instantiate_if_expression(
                            conditional,
                            result.clone(),
                            Some(result), // TODO: this is a really bad hack and we should not do this
                            result_span,
                            self.return_type_id, // TODO: figure out if this argument matters or not
                            ctx.self_type()
                        ),
                        continue,
                        warnings,
                        errors
                    )
                }
                (Some(prev_if_exp), None) => {
                    let conditional = TyExpression {
                        expression: TyExpressionVariant::Literal(Literal::Boolean(true)),
                        return_type: insert_type(TypeInfo::Boolean),
                        is_constant: IsConstant::No,
                        span: result_span.clone(),
                    };
                    check!(
                        instantiate_if_expression(
                            conditional,
                            result,
                            Some(prev_if_exp),
                            result_span,
                            self.return_type_id,
                            ctx.self_type()
                        ),
                        continue,
                        warnings,
                        errors
                    )
                }
                (Some(prev_if_exp), Some(conditional)) => {
                    check!(
                        instantiate_if_expression(
                            conditional,
                            result,
                            Some(prev_if_exp),
                            result_span,
                            self.return_type_id,
                            ctx.self_type()
                        ),
                        continue,
                        warnings,
                        errors
                    )
                }
            });
        }

        // return!
        match typed_if_exp {
            None => {
                errors.push(CompileError::Internal(
                    "unable to convert match exp to if exp",
                    self.span,
                ));
                err(warnings, errors)
            }
            Some(typed_if_exp) => ok(typed_if_exp, warnings, errors),
        }
    }
}
