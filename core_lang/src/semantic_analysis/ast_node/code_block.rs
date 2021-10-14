use super::*;
use crate::build_config::BuildConfig;
use crate::control_flow_analysis::ControlFlowGraph;
use crate::types::MaybeResolvedType;
use crate::CodeBlock;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub(crate) struct TypedCodeBlock<'sc> {
    pub(crate) contents: Vec<TypedAstNode<'sc>>,
    pub(crate) whole_block_span: Span<'sc>,
}

impl<'sc> TypedCodeBlock<'sc> {
    pub(crate) fn replace_self_types(&self, _self_type: &MaybeResolvedType<'sc>) -> Self {
        // TODO recursively replace all self types in the block
        self.clone()
    }
    pub(crate) fn type_check(
        other: CodeBlock<'sc>,
        namespace: &Namespace<'sc>,
        // this is for the return or implicit return
        type_annotation: Option<MaybeResolvedType<'sc>>,
        help_text: impl Into<String> + Clone,
        self_type: &MaybeResolvedType<'sc>,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph<'sc>,
        dependency_graph: &mut HashMap<String, HashSet<String>>,
    ) -> CompileResult<'sc, (Self, Option<MaybeResolvedType<'sc>>)> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Mutable clone, because the interior of a code block must not change the surrounding
        // namespace.
        let mut local_namespace = namespace.clone();
        let evaluated_contents = other
            .contents
            .iter()
            .filter_map(|node| {
                TypedAstNode::type_check(
                    node.clone(),
                    &mut local_namespace,
                    type_annotation.clone(),
                    help_text.clone(),
                    self_type,
                    build_config,
                    dead_code_graph,
                    dependency_graph,
                )
                .ok(&mut warnings, &mut errors)
            })
            .collect::<Vec<TypedAstNode<'sc>>>();

        let implicit_return_span = other
            .contents
            .iter()
            .find_map(|x| match &x.content {
                AstNodeContent::ImplicitReturnExpression(expr) => Some(Some(expr.span())),
                _ => None,
            })
            .flatten();

        // find the implicit return, if any, and use it as the code block's return type.
        // The fact that there is at most one implicit return is an invariant held by the core_lang.
        let return_type = evaluated_contents.iter().find_map(|x| match x {
            TypedAstNode {
                content:
                    TypedAstNodeContent::ImplicitReturnExpression(TypedExpression {
                        ref return_type,
                        ..
                    }),
                ..
            } => Some(return_type.clone()),
            _ => None,
        });
        if let Some(ref return_type) = return_type {
            if let Some(type_annotation) = type_annotation {
                let convertability = return_type.is_convertible(
                    &type_annotation,
                    implicit_return_span.unwrap_or_else(|| other.whole_block_span.clone()),
                    help_text,
                );
                match convertability {
                    Ok(warning) => {
                        if let Some(warning) = warning {
                            warnings.push(CompileWarning {
                                warning_content: warning,
                                span: other.whole_block_span.clone(),
                            });
                        }
                    }
                    Err(err) => {
                        errors.push(err.into());
                    }
                }
            }
        }

        ok(
            (
                TypedCodeBlock {
                    contents: evaluated_contents,
                    whole_block_span: other.whole_block_span,
                },
                return_type,
            ),
            warnings,
            errors,
        )
    }
}
