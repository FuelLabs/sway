use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::Span;

use crate::{
    language::{parsed::*, ty, *},
    semantic_analysis::{
        ast_node::expression::typed_expression::{
            instantiate_if_expression, instantiate_lazy_operator,
        },
        TypeCheckContext,
    },
    CompileError, TypeInfo,
};

impl ty::TyMatchExpression {
    pub(crate) fn type_check(
        handler: &Handler,
        ctx: TypeCheckContext,
        typed_value: ty::TyExpression,
        branches: Vec<MatchBranch>,
        span: Span,
    ) -> Result<(ty::TyMatchExpression, Vec<ty::TyScrutinee>), ErrorEmitted> {
        // type check all of the branches
        let mut typed_branches = vec![];
        let mut typed_scrutinees = vec![];
        let mut ctx =
            ctx.with_help_text("all branches of a match statement must return the same type");

        handler.scope(|handler| {
            for branch in branches.into_iter() {
                let (typed_branch, typed_scrutinee) = match ty::TyMatchBranch::type_check(
                    handler,
                    ctx.by_ref(),
                    &typed_value,
                    branch,
                ) {
                    Ok(res) => res,
                    Err(_) => continue,
                };
                typed_branches.push(typed_branch);
                typed_scrutinees.push(typed_scrutinee);
            }
            Ok(())
        })?;

        let typed_exp = ty::TyMatchExpression {
            value_type_id: typed_value.return_type,
            branches: typed_branches,
            return_type_id: ctx.type_annotation(),
            span,
        };
        Ok((typed_exp, typed_scrutinees))
    }

    pub(crate) fn convert_to_typed_if_expression(
        self,
        handler: &Handler,
        mut ctx: TypeCheckContext,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();

        // create the typed if expression object that we will be building on to
        let mut typed_if_exp: Option<ty::TyExpression> = None;

        // for every branch of the match expression
        for ty::TyMatchBranch { cnf, result, .. } in self.branches.into_iter().rev() {
            let mut conj_conditional: Option<ty::TyExpression> = None;

            for disjunction in cnf.into_iter().rev() {
                // create the conditional that will act as the conditional for the if statement, in reverse
                let mut disj_conditional: Option<ty::TyExpression> = None;
                for (left_req, right_req) in disjunction.into_iter().rev() {
                    let joined_span = Span::join(left_req.span.clone(), right_req.span.clone());
                    let args = vec![left_req, right_req];
                    let new_condition = match ty::TyExpression::core_ops_eq(
                        handler,
                        ctx.by_ref(),
                        args,
                        joined_span,
                    ) {
                        Ok(res) => res,
                        Err(_) => {
                            continue;
                        }
                    };

                    disj_conditional = Some(match disj_conditional {
                        Some(inner_condition) => {
                            let joined_span = Span::join(
                                inner_condition.span.clone(),
                                new_condition.span.clone(),
                            );
                            instantiate_lazy_operator(
                                LazyOp::Or,
                                new_condition,
                                inner_condition,
                                type_engine.insert(engines, TypeInfo::Boolean),
                                joined_span,
                            )
                        }
                        None => new_condition,
                    });
                }

                let new_condition = disj_conditional;
                conj_conditional = match (conj_conditional, new_condition) {
                    (Some(inner_condition), Some(new_condition)) => {
                        let joined_span =
                            Span::join(inner_condition.span.clone(), new_condition.span.clone());
                        Some(instantiate_lazy_operator(
                            LazyOp::And,
                            new_condition,
                            inner_condition,
                            type_engine.insert(engines, TypeInfo::Boolean),
                            joined_span,
                        ))
                    }
                    (exp @ Some(_), None) | (None, exp @ Some(_)) => exp,
                    (None, None) => None,
                }
            }

            // add to the if expression that we are building using the result component
            // of the match branch and using the conditional that we just built
            let result_span = result.span.clone();
            typed_if_exp = Some(match (typed_if_exp.clone(), conj_conditional) {
                (None, None) => result,
                (None, Some(conditional)) => {
                    // TODO: figure out if this argument matters or not
                    let ctx = ctx.by_ref().with_type_annotation(self.return_type_id);
                    match instantiate_if_expression(
                        handler,
                        ctx,
                        conditional,
                        result.clone(),
                        Some(result), // TODO: this is a really bad hack and we should not do this
                        result_span,
                    ) {
                        Ok(res) => res,
                        Err(_) => {
                            continue;
                        }
                    }
                }
                (Some(prev_if_exp), None) => {
                    let ctx = ctx.by_ref().with_type_annotation(self.return_type_id);
                    let conditional = ty::TyExpression {
                        expression: ty::TyExpressionVariant::Literal(Literal::Boolean(true)),
                        return_type: type_engine.insert(engines, TypeInfo::Boolean),
                        span: result_span.clone(),
                    };
                    match instantiate_if_expression(
                        handler,
                        ctx,
                        conditional,
                        result,
                        Some(prev_if_exp),
                        result_span,
                    ) {
                        Ok(res) => res,
                        Err(_) => {
                            continue;
                        }
                    }
                }
                (Some(prev_if_exp), Some(conditional)) => {
                    let ctx = ctx.by_ref().with_type_annotation(self.return_type_id);
                    match instantiate_if_expression(
                        handler,
                        ctx,
                        conditional,
                        result,
                        Some(prev_if_exp),
                        result_span,
                    ) {
                        Ok(res) => res,
                        Err(_) => {
                            continue;
                        }
                    }
                }
            });
        }

        // return!
        match typed_if_exp {
            None => {
                // If the type that we are matching on does not have a valid
                // constructor, then it is expected that this algorithm finds a
                // "None". This is because the user has not provided any
                // branches in the match expression because the type cannot be
                // constructed or matched upon. In this case, we manually create
                // a typed expression that is equivalent to
                // "if true { implicit_return }" where the implicit_return type is manually set
                // to be the return type of this typed match expression object.
                //
                // NOTE: This manual construction of the expression can (and
                // most likely will) lead to an otherwise improperly typed
                // expression, in most cases.
                if !type_engine
                    .get(self.value_type_id)
                    .has_valid_constructor(decl_engine)
                {
                    let condition = ty::TyExpression {
                        expression: ty::TyExpressionVariant::Literal(Literal::Boolean(true)),
                        return_type: type_engine.insert(engines, TypeInfo::Boolean),
                        span: self.span.clone(),
                    };
                    let then_exp = ty::TyExpression {
                        expression: ty::TyExpressionVariant::Tuple { fields: vec![] },
                        return_type: self.return_type_id,
                        span: self.span.clone(),
                    };
                    let inner_exp = ty::TyExpressionVariant::IfExp {
                        condition: Box::new(condition),
                        then: Box::new(then_exp.clone()),
                        r#else: Option::Some(Box::new(then_exp)),
                    };
                    let typed_if_exp = ty::TyExpression {
                        expression: inner_exp,
                        return_type: self.return_type_id,
                        span: self.span,
                    };
                    return Ok(typed_if_exp);
                }

                Err(handler.emit_err(CompileError::Internal(
                    "unable to convert match exp to if exp",
                    self.span,
                )))
            }
            Some(typed_if_exp) => Ok(typed_if_exp),
        }
    }
}
