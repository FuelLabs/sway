use super::*;
use crate::language::{parsed::CodeBlock, ty};

impl ty::TyCodeBlock {
    pub(crate) fn type_check(
        mut ctx: TypeCheckContext,
        code_block: CodeBlock,
    ) -> CompileResult<(Self, TypeId)> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let decl_engine = ctx.decl_engine;

        // Create a temp namespace for checking within the code block scope.
        let mut code_block_namespace = ctx.namespace.clone();
        let evaluated_contents = code_block
            .contents
            .iter()
            .filter_map(|node| {
                let ctx = ctx.by_ref().scoped(&mut code_block_namespace);
                ty::TyAstNode::type_check(ctx, node.clone()).ok(&mut warnings, &mut errors)
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
                        Ident::new_with_override("core", span.clone()),
                        Ident::new_with_override("never", span.clone()),
                    ];
                    let never_ident = Ident::new_with_override("Never", span.clone());

                    let never_decl_opt = ctx
                        .namespace
                        .root()
                        .resolve_symbol(&never_mod_path, &never_ident)
                        .value;

                    if let Some(ty::TyDeclaration::EnumDeclaration {
                        decl_id: never_decl_id,
                        ..
                    }) = never_decl_opt
                    {
                        if let Ok(never_decl) = decl_engine.get_enum(never_decl_id, &span) {
                            return never_decl.create_type_id(ctx.engines());
                        }
                    }

                    ctx.type_engine.insert(decl_engine, TypeInfo::Unknown)
                } else {
                    ctx.type_engine
                        .insert(decl_engine, TypeInfo::Tuple(Vec::new()))
                }
            });

        append!(ctx.unify_with_self(block_type, &span), warnings, errors);

        let typed_code_block = ty::TyCodeBlock {
            contents: evaluated_contents,
        };
        ok((typed_code_block, block_type), warnings, errors)
    }
}
