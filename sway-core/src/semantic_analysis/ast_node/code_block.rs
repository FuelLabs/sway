use super::*;
use crate::CodeBlock;

#[derive(Clone, Debug)]
pub struct TypedCodeBlock {
    pub contents: Vec<TypedAstNode>,
}

impl PartialEq for CompileWrapper<'_, TypedCodeBlock> {
    fn eq(&self, other: &Self) -> bool {
        let CompileWrapper {
            inner: me,
            declaration_engine: de,
        } = self;
        let CompileWrapper { inner: them, .. } = other;
        me.contents.wrap(de) == them.contents.wrap(de)
    }
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

        // find the implicit return, if any, and use it as the code block's return type.
        // The fact that there is at most one implicit return is an invariant held by the parser.
        let return_type = evaluated_contents.iter().find_map(|x| match x {
            TypedAstNode {
                content:
                    TypedAstNodeContent::ImplicitReturnExpression(TypedExpression {
                        ref return_type,
                        ..
                    }),
                ..
            } if !x.deterministically_aborts() => Some(*return_type),
            _ => None,
        });

        if let Some(return_type) = return_type {
            let span = implicit_return_span.unwrap_or_else(|| code_block.whole_block_span.clone());
            let (mut new_warnings, new_errors) = ctx.unify_with_self(return_type, &span);
            warnings.append(&mut new_warnings);
            errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
            // The annotation will result in a cast, so set the return type accordingly.
        }

        let typed_code_block = TypedCodeBlock {
            contents: evaluated_contents,
        };
        let type_id = return_type.unwrap_or_else(|| insert_type(TypeInfo::Tuple(Vec::new())));
        ok((typed_code_block, type_id), warnings, errors)
    }
}
