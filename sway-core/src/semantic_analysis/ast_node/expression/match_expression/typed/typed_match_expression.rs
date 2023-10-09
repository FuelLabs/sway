use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::Span;

use crate::{
    language::{parsed::*, ty, *},
    semantic_analysis::{
        ast_node::expression::typed_expression::instantiate_if_expression,
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

        handler.scope(|handler| {
            // for every branch of the match expression, in reverse
            for ty::TyMatchBranch { if_condition, result, .. } in self.branches.into_iter().rev() {
                // add to the if expression that we are building using the result component
                // of the match branch and using the if condition coming from the branch
                let result_span = result.span.clone();
                typed_if_exp = Some(match (typed_if_exp.clone(), if_condition) {
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

            Ok(())
        })?;

        // return!
        match typed_if_exp {
            None => {
                // If the type that we are matching on does not have a valid
                // constructor, then it is expected that the above algorithm finds a
                // `None`. This is because the user has not provided any
                // branches in the match expression because the type cannot be
                // constructed or matched upon. In this case, we manually create
                // a typed expression that is equivalent to
                // `if true { implicit_return }` where the implicit_return type is manually set
                // to be the return type of this typed match expression object.
                //
                // An example of such matching is when matching an empty enum.
                // For an example, see the "match_expressions_empty_enums" test.
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
            },
            Some(typed_if_exp) => Ok(typed_if_exp),
        }
    }
}
