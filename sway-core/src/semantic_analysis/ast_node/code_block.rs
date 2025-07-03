use super::*;
use crate::language::{
    parsed::CodeBlock,
    ty::{self, TyAstNodeContent, TyCodeBlock},
};

impl ty::TyCodeBlock {
    pub(crate) fn collect(
        handler: &Handler,
        engines: &Engines,
        ctx: &mut SymbolCollectionContext,
        code_block: &CodeBlock,
    ) -> Result<(), ErrorEmitted> {
        let _ = ctx.scoped(
            engines,
            code_block.whole_block_span.clone(),
            None,
            |scoped_ctx| {
                let _ = code_block
                    .contents
                    .iter()
                    .map(|node| ty::TyAstNode::collect(handler, engines, scoped_ctx, node))
                    .filter_map(|res| res.ok())
                    .collect::<Vec<_>>();
                Ok(())
            },
        );
        Ok(())
    }

    pub(crate) fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        code_block: &CodeBlock,
        is_root: bool,
    ) -> Result<Self, ErrorEmitted> {
        if !is_root {
            let code_block_result =
                ctx.by_ref()
                    .scoped(handler, Some(code_block.span()), |ctx| {
                        let evaluated_contents = code_block
                            .contents
                            .iter()
                            .filter_map(|node| {
                                ty::TyAstNode::type_check(handler, ctx.by_ref(), node).ok()
                            })
                            .collect::<Vec<ty::TyAstNode>>();
                        Ok(ty::TyCodeBlock {
                            contents: evaluated_contents,
                            whole_block_span: code_block.whole_block_span.clone(),
                        })
                    })?;

            return Ok(code_block_result);
        }

        ctx.engines.te().clear_unifications();
        ctx.namespace()
            .current_module()
            .current_lexical_scope()
            .items
            .clear_symbols_unique_while_collecting_unifications();

        // We are typechecking the code block AST nodes twice.
        // The first pass does all the unifications to the variables types.
        // In the second pass we use the previous_namespace on variable declaration to unify directly with the result of the first pass.
        // This is required to fix the test case numeric_type_propagation and issue #6371
        ctx.by_ref()
            .with_collecting_unifications()
            .with_code_block_first_pass(true)
            .scoped(handler, Some(code_block.span()), |ctx| {
                code_block.contents.iter().for_each(|node| {
                    ty::TyAstNode::type_check(&Handler::default(), ctx.by_ref(), node).ok();
                });
                Ok(())
            })?;

        ctx.engines.te().reapply_unifications(ctx.engines(), 0);

        ctx.by_ref()
            .scoped(handler, Some(code_block.span()), |ctx| {
                let evaluated_contents = code_block
                    .contents
                    .iter()
                    .filter_map(|node| ty::TyAstNode::type_check(handler, ctx.by_ref(), node).ok())
                    .collect::<Vec<ty::TyAstNode>>();

                Ok(ty::TyCodeBlock {
                    contents: evaluated_contents,
                    whole_block_span: code_block.whole_block_span.clone(),
                })
            })
    }

    pub fn compute_return_type_and_span(
        ctx: &TypeCheckContext,
        code_block: &TyCodeBlock,
    ) -> (TypeId, Span) {
        let implicit_return_span = code_block
            .contents
            .iter()
            .find_map(|x| match &x.content {
                TyAstNodeContent::Expression(ty::TyExpression {
                    expression: ty::TyExpressionVariant::ImplicitReturn(expr),
                    ..
                }) => Some(Some(expr.span.clone())),
                _ => None,
            })
            .flatten();
        let span = implicit_return_span.unwrap_or_else(|| code_block.whole_block_span.clone());

        let block_type = code_block
            .contents
            .iter()
            .find_map(|node| {
                match node {
                    // If an ast node of the block returns, panics, breaks, or continues then the whole block should have `Never` as return type.
                    ty::TyAstNode {
                        content:
                            ty::TyAstNodeContent::Expression(ty::TyExpression {
                                expression:
                                    ty::TyExpressionVariant::Return(_)
                                    | ty::TyExpressionVariant::Panic(_)
                                    | ty::TyExpressionVariant::Break
                                    | ty::TyExpressionVariant::Continue,
                                ..
                            }),
                        ..
                    } => Some(ctx.engines.te().id_of_never()),
                    // Find the implicit return, if any, and use it as the code block's return type.
                    // The fact that there is at most one implicit return is an invariant held by the parser.
                    ty::TyAstNode {
                        content:
                            ty::TyAstNodeContent::Expression(ty::TyExpression {
                                expression: ty::TyExpressionVariant::ImplicitReturn(_expr),
                                return_type,
                                ..
                            }),
                        ..
                    } => Some(*return_type),
                    // If an ast node of the block has Never as return type then the whole block should have Never as return type.
                    ty::TyAstNode {
                        content:
                            ty::TyAstNodeContent::Expression(ty::TyExpression { return_type, .. }),
                        ..
                    } => {
                        if matches!(*ctx.engines.te().get(*return_type), TypeInfo::Never) {
                            Some(*return_type)
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            })
            .unwrap_or_else(|| ctx.engines.te().id_of_unit());

        (block_type, span)
    }
}

impl TypeCheckAnalysis for ty::TyCodeBlock {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        for node in self.contents.iter() {
            node.type_check_analyze(handler, ctx)?;
        }
        Ok(())
    }
}

impl TypeCheckFinalization for ty::TyCodeBlock {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        handler.scope(|handler| {
            for node in self.contents.iter_mut() {
                let _ = node.type_check_finalize(handler, ctx);
            }
            Ok(())
        })
    }
}
