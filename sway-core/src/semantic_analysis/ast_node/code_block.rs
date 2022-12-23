use super::*;
use crate::language::{parsed::CodeBlock, ty};

impl ty::TyCodeBlock {
    pub(crate) fn type_check(
        mut ctx: TypeCheckContext,
        code_block: CodeBlock,
    ) -> CompileResult<(Self, TypeId)> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let type_engine = ctx.type_engine;
        let declaration_engine = ctx.declaration_engine;
        // let engines = ctx.engines();

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
        let block_type = evaluated_contents
            .iter()
            .find_map(|node| {
                if node.deterministically_aborts(declaration_engine, true) {
                    Some(type_engine.insert_type(declaration_engine, TypeInfo::Unknown))
                } else {
                    match node {
                        ty::TyAstNode {
                            content:
                                ty::TyAstNodeContent::ImplicitReturnExpression(ty::TyExpression {
                                    ref return_type,
                                    ..
                                }),
                            ..
                        } => {
                            // if !type_engine.check_if_types_can_be_coerced(
                            //     declaration_engine,
                            //     *return_type,
                            //     ctx.type_annotation(),
                            // ) {
                            //     errors.push(CompileError::TypeError(TypeError::MismatchedType {
                            //         expected: engines.help_out(ctx.type_annotation()).to_string(),
                            //         received: engines.help_out(return_type).to_string(),
                            //         help_text: "Implicit return must match up with block's type."
                            //             .to_string(),
                            //         span: span.clone(),
                            //     }));
                            // }
                            Some(*return_type)
                        }
                        _ => None,
                    }
                }
            })
            .unwrap_or_else(|| {
                type_engine.insert_type(declaration_engine, TypeInfo::Tuple(Vec::new()))
            });

        append!(ctx.unify_with_self(block_type, &span), warnings, errors);

        let typed_code_block = ty::TyCodeBlock {
            contents: evaluated_contents,
        };

        ok((typed_code_block, block_type), warnings, errors)
    }
}
