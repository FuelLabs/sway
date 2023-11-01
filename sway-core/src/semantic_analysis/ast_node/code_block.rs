use super::*;
use crate::{
    decl_engine::DeclRef,
    language::{
        parsed::CodeBlock,
        ty::{self, TyAstNodeContent, TyCodeBlock},
    },
};

impl ty::TyCodeBlock {
    pub(crate) fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        code_block: &CodeBlock,
    ) -> Result<Self, ErrorEmitted> {
        // Create a temp namespace for checking within the code block scope.
        let mut code_block_namespace = ctx.namespace.clone();
        let evaluated_contents = code_block
            .contents
            .iter()
            .filter_map(|node| {
                let ctx = ctx.by_ref().scoped(&mut code_block_namespace);
                ty::TyAstNode::type_check(handler, ctx, node.clone()).ok()
            })
            .collect::<Vec<ty::TyAstNode>>();

        Ok(ty::TyCodeBlock {
            contents: evaluated_contents,
            whole_block_span: code_block.whole_block_span.clone(),
        })
    }

    pub fn compute_return_type_and_span(
        ctx: &TypeCheckContext,
        code_block: &TyCodeBlock,
    ) -> (TypeId, Span) {
        let engines = ctx.engines();
        let decl_engine = engines.de();

        let implicit_return_span = code_block
            .contents
            .iter()
            .find_map(|x| match &x.content {
                TyAstNodeContent::ImplicitReturnExpression(expr) => Some(Some(expr.span.clone())),
                _ => None,
            })
            .flatten();
        let span = implicit_return_span.unwrap_or_else(|| code_block.whole_block_span.clone());

        // find the implicit return, if any, and use it as the code block's return type.
        // The fact that there is at most one implicit return is an invariant held by the parser.
        // If any node diverges then the entire block has unknown type.
        let mut node_deterministically_aborts = false;
        let block_type = code_block
            .contents
            .iter()
            .find_map(|node| {
                if node.deterministically_aborts(decl_engine, true) {
                    node_deterministically_aborts = true;
                };
                match node {
                    ty::TyAstNode {
                        content:
                            ty::TyAstNodeContent::ImplicitReturnExpression(ty::TyExpression {
                                ref return_type,
                                ..
                            }),
                        ..
                    } => Some(*return_type),
                    _ => None,
                }
            })
            .unwrap_or_else(|| {
                if node_deterministically_aborts {
                    let never_mod_path = vec![
                        Ident::new_with_override("core".into(), span.clone()),
                        Ident::new_with_override("never".into(), span.clone()),
                    ];
                    let never_ident = Ident::new_with_override("Never".into(), span.clone());

                    let never_decl_opt = ctx
                        .namespace
                        .root()
                        .resolve_symbol(
                            &Handler::default(),
                            engines,
                            &never_mod_path,
                            &never_ident,
                            None,
                        )
                        .ok();

                    if let Some(ty::TyDecl::EnumDecl(ty::EnumDecl {
                        name,
                        decl_id,
                        subst_list: _,
                        decl_span,
                    })) = never_decl_opt
                    {
                        return ctx.engines().te().insert(
                            engines,
                            TypeInfo::Enum(DeclRef::new(name.clone(), decl_id, decl_span.clone())),
                            name.span().source_id(),
                        );
                    }

                    ctx.engines.te().insert(engines, TypeInfo::Unknown, None)
                } else {
                    ctx.engines
                        .te()
                        .insert(engines, TypeInfo::Tuple(Vec::new()), span.source_id())
                }
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

impl TypeCheckUnification for ty::TyCodeBlock {
    fn type_check_unify(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckUnificationContext,
    ) -> Result<(), ErrorEmitted> {
        handler.scope(|handler| {
            let type_check_ctx = &ctx.type_check_ctx;
            let (block_implicit_return, span) =
                TyCodeBlock::compute_return_type_and_span(type_check_ctx, self);
            let return_type_id = match ctx.type_id {
                Some(type_id) => type_id,
                None => block_implicit_return,
            };
            type_check_ctx.unify_with_type_annotation(handler, return_type_id, &span);
            Ok(())
        })
    }
}
