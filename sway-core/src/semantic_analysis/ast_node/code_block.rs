use super::*;
use crate::{
    decl_engine::DeclRef,
    language::{parsed::CodeBlock, ty},
};

impl ty::TyCodeBlock {
    pub(crate) fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        code_block: CodeBlock,
    ) -> Result<(Self, TypeId), ErrorEmitted> {
        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();

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

        let implicit_return_span = code_block
            .contents
            .iter()
            .find_map(|x| match &x.content {
                AstNodeContent::ImplicitReturnExpression(expr) => Some(Some(expr.span())),
                _ => None,
            })
            .flatten();
        let span = implicit_return_span.unwrap_or_else(|| code_block.whole_block_span.clone());

        // find the implicit return, if any, and use it as the code block's return type.
        // The fact that there is at most one implicit return is an invariant held by the parser.
        // If any node diverges then the entire block has unknown type.
        let mut node_deterministically_aborts = false;
        let block_type = evaluated_contents
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
                        .resolve_symbol(&Handler::default(), engines, &never_mod_path, &never_ident)
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
                        );
                    }

                    ctx.engines.te().insert(engines, TypeInfo::Unknown)
                } else {
                    ctx.engines
                        .te()
                        .insert(engines, TypeInfo::Tuple(Vec::new()))
                }
            });

        ctx.unify_with_self(handler, block_type, &span);

        let typed_code_block = ty::TyCodeBlock {
            contents: evaluated_contents,
        };
        Ok((typed_code_block, block_type))
    }
}
