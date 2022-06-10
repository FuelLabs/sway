use super::*;
use crate::semantic_analysis::{ast_node::Mode, TypeCheckArguments};
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
        arguments: TypeCheckArguments<'_, CodeBlock>,
    ) -> CompileResult<(Self, TypeId)> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let TypeCheckArguments {
            checkee: other,
            namespace,
            return_type_annotation,
            help_text,
            self_type,
            opts,
            ..
        } = arguments;

        // Create a temp namespace for checking within the code block scope.
        let mut code_block_namespace = namespace.clone();
        let evaluated_contents = other
            .contents
            .iter()
            .filter_map(|node| {
                TypedAstNode::type_check(TypeCheckArguments {
                    checkee: node.clone(),
                    namespace: &mut code_block_namespace,
                    return_type_annotation,
                    help_text,
                    self_type,
                    mode: Mode::NonAbi,
                    opts,
                })
                .ok(&mut warnings, &mut errors)
            })
            .collect::<Vec<TypedAstNode>>();

        let implicit_return_span = other
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
            } => Some(*return_type),
            _ => None,
        });

        if let Some(return_type) = return_type {
            let (mut new_warnings, new_errors) = unify_with_self(
                return_type,
                return_type_annotation,
                self_type,
                &implicit_return_span.unwrap_or_else(|| other.whole_block_span.clone()),
                help_text,
            );
            warnings.append(&mut new_warnings);
            errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
            // The annotation will result in a cast, so set the return type accordingly.
        }

        ok(
            (
                TypedCodeBlock {
                    contents: evaluated_contents,
                },
                return_type.unwrap_or_else(|| insert_type(TypeInfo::Tuple(Vec::new()))),
            ),
            warnings,
            errors,
        )
    }
}
