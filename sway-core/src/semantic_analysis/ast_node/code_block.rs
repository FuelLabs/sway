use super::*;
use crate::language::{
    parsed::CodeBlock,
    ty::{self, TyAstNodeContent, TyCodeBlock},
};

impl ty::TyCodeBlock {
    pub(crate) fn type_check(
        handler: &Handler,
        ctx: TypeCheckContext,
        code_block: &CodeBlock,
    ) -> Result<Self, ErrorEmitted> {
        ctx.scoped(|mut ctx| {
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
        let engines = ctx.engines();

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
                    // If an ast node of the block returns, breaks, or continues then the whole block should have Never as return type.
                    ty::TyAstNode {
                        content:
                            ty::TyAstNodeContent::Expression(ty::TyExpression {
                                expression:
                                    ty::TyExpressionVariant::Return(_)
                                    | ty::TyExpressionVariant::Break
                                    | ty::TyExpressionVariant::Continue,
                                ..
                            }),
                        ..
                    } => Some(
                        ctx.engines
                            .te()
                            .insert(engines, TypeInfo::Never, span.source_id()),
                    ),
                    // find the implicit return, if any, and use it as the code block's return type.
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
            .unwrap_or_else(|| {
                ctx.engines
                    .te()
                    .insert(engines, TypeInfo::Tuple(Vec::new()), span.source_id())
            });

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
