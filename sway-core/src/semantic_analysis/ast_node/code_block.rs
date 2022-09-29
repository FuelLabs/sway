use super::*;
use crate::CodeBlock;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedCodeBlock {
    pub contents: Vec<TypedAstNode>,
}

impl CopyTypes for TypedCodeBlock {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.contents
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
    }
}

impl DeterministicallyAborts for TypedCodeBlock {
    fn deterministically_aborts(&self) -> bool {
        self.contents.iter().any(|x| x.deterministically_aborts())
    }
}

impl TypedCodeBlock {
    pub(crate) fn type_check(
        mut ctx: TypeCheckContext,
        code_block: CodeBlock,
    ) -> CompileResult<(Self, TypeId)> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Create a temp namespace for checking within the code block scope.
        let mut code_block_namespace = ctx.namespace.clone();
        let evaluated_contents = code_block
            .contents
            .iter()
            .filter_map(|node| {
                let ctx = ctx.by_ref().scoped(&mut code_block_namespace);
                TypedAstNode::type_check(ctx, node.clone()).ok(&mut warnings, &mut errors)
            })
            .collect::<Vec<TypedAstNode>>();

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
                if node.deterministically_aborts() {
                    Some(insert_type(TypeInfo::Unknown))
                } else {
                    match node {
                        TypedAstNode {
                            content:
                                TypedAstNodeContent::ImplicitReturnExpression(TypedExpression {
                                    ref return_type,
                                    ..
                                }),
                            ..
                        } => Some(*return_type),
                        _ => None,
                    }
                }
            })
            .unwrap_or_else(|| insert_type(TypeInfo::Tuple(Vec::new())));

        append!(
            ctx.unify_with_type_annotation_and_self(block_type, &span),
            warnings,
            errors
        );

        let typed_code_block = TypedCodeBlock {
            contents: evaluated_contents,
        };
        ok((typed_code_block, block_type), warnings, errors)
    }
}
