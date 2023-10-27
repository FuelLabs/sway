use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{Span, Spanned};

use crate::{
    compiler_generated::INVALID_DESUGARED_MATCHED_EXPRESSION_SIGNAL,
    language::{parsed::*, ty},
    semantic_analysis::{
        ast_node::expression::typed_expression::instantiate_if_expression,
        expression::match_expression::typed::instantiate::Instantiate, TypeCheckContext,
    },
    CompileError, TypeId,
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
        let instantiate = Instantiate::new(ctx.engines, self.span.clone());

        if self.branches.is_empty() {
            return instantiate_if_expression_for_empty_match_expression(
                handler,
                ctx,
                &instantiate,
                self.value_type_id,
                self.return_type_id,
                self.span.clone(),
            );
        }

        let typed_if_exp = handler.scope(|handler| {
            // The typed if expression object that we will be building on to.
            // We will do it bottom up, starting from the final `else`.
            let mut typed_if_exp = None;

            // For every branch, bottom-up, means in reverse.
            for ty::TyMatchBranch {
                matched_or_variant_index_vars,
                condition,
                result,
                ..
            } in self.branches.into_iter().rev()
            {
                // If we are instantiating the final `else` block.
                if typed_if_exp.is_none() {
                    // If the last match arm is a catch-all arm make its result the final else.
                    // Note that this will always be the case with `if let` expressions that
                    // desugar to match expressions.
                    if condition.is_none() {
                        typed_if_exp = Some(result);
                        continue; // Last branch added, move to the previous one.
                    } else {
                        // Otherwise instantiate the final `__revert`.
                        let final_revert = instantiate.code_block_with_implicit_return_revert(
                            INVALID_DESUGARED_MATCHED_EXPRESSION_SIGNAL,
                        );

                        typed_if_exp = Some(final_revert);
                        // Continue with adding the last branch.
                    };
                }

                // Create a new namespace for this branch result.
                let ctx = ctx.by_ref().with_type_annotation(self.return_type_id);
                let mut namespace = ctx.namespace.clone();
                let mut branch_ctx = ctx.scoped(&mut namespace);

                let result_span = result.span.clone();
                let condition = condition.unwrap_or(instantiate.boolean_literal(true));

                let if_exp = match instantiate_if_expression(
                    handler,
                    branch_ctx.by_ref(),
                    condition,
                    result,
                    Some(
                        typed_if_exp
                            .clone()
                            .expect("The previously created expression exist at this point."),
                    ), // Put the previous if into else.
                    result_span.clone(),
                ) {
                    Ok(if_exp) => if_exp,
                    Err(_) => {
                        continue;
                    }
                };

                typed_if_exp = if matched_or_variant_index_vars.is_empty() {
                    // No OR variants with vars. We just have to instantiate the if expression.
                    Some(if_exp)
                } else {
                    // We have matched OR variant index vars.
                    // We need to add them to the block before the if expression.
                    // The resulting `typed_if_exp` in this case is actually not
                    // an if expression but rather a code block.
                    let mut code_block_contents: Vec<ty::TyAstNode> = vec![];

                    for (var_ident, var_body) in matched_or_variant_index_vars {
                        let var_decl = instantiate.var_decl(var_ident.clone(), var_body);
                        let span = var_ident.span();
                        let _ = branch_ctx.insert_symbol(handler, var_ident, var_decl.clone());
                        code_block_contents.push(ty::TyAstNode {
                            content: ty::TyAstNodeContent::Declaration(var_decl),
                            span,
                        });
                    }

                    code_block_contents.push(ty::TyAstNode {
                        content: ty::TyAstNodeContent::ImplicitReturnExpression(if_exp),
                        span: result_span.clone(),
                    });

                    Some(ty::TyExpression {
                        expression: ty::TyExpressionVariant::CodeBlock(ty::TyCodeBlock {
                            whole_block_span: Span::dummy(),
                            contents: code_block_contents,
                        }),
                        return_type: self.return_type_id,
                        span: result_span.clone(),
                    })
                }
            }

            Ok(typed_if_exp.expect("The expression exists because we have at least one branch."))
        })?;

        return Ok(typed_if_exp);

        fn instantiate_if_expression_for_empty_match_expression(
            handler: &Handler,
            ctx: TypeCheckContext,
            instantiate: &Instantiate,
            value_type_id: TypeId,
            return_type_id: TypeId,
            span: Span,
        ) -> Result<ty::TyExpression, ErrorEmitted> {
            let type_engine = ctx.engines.te();
            let decl_engine = ctx.engines.de();

            // An empty match expression can happen only if the type we
            // are matching on does not have a valid constructor.
            // Otherwise, the match expression must be exhaustive, means
            // it must have at least one match arm.
            // In this case, we manually create a typed expression that is equivalent to
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
                .get(value_type_id)
                .has_valid_constructor(decl_engine)
            {
                let condition = instantiate.boolean_literal(true);
                let then_exp = ty::TyExpression {
                    expression: ty::TyExpressionVariant::Tuple { fields: vec![] },
                    return_type: return_type_id,
                    span: instantiate.dummy_span(),
                };
                let inner_exp = ty::TyExpressionVariant::IfExp {
                    condition: Box::new(condition),
                    then: Box::new(then_exp.clone()),
                    r#else: Option::Some(Box::new(then_exp)),
                };
                let typed_if_exp = ty::TyExpression {
                    expression: inner_exp,
                    return_type: return_type_id,
                    span: instantiate.dummy_span(),
                };

                return Ok(typed_if_exp);
            }

            Err(handler.emit_err(CompileError::Internal(
                "unable to convert match exp to if exp",
                span,
            )))
        }
    }
}
