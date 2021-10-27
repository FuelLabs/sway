use super::*;
use crate::build_config::BuildConfig;
use crate::control_flow_analysis::ControlFlowGraph;
use crate::type_engine::{TypeEngine, TYPE_ENGINE};
use crate::CodeBlock;

#[derive(Clone, Debug)]
pub(crate) struct TypedCodeBlock<'sc> {
    pub(crate) contents: Vec<TypedAstNode<'sc>>,
    pub(crate) whole_block_span: Span<'sc>,
}

impl<'sc> TypedCodeBlock<'sc> {
    pub(crate) fn type_check(
        other: CodeBlock<'sc>,
        namespace: &Namespace<'sc>,
        // this is for the return or implicit return
        type_annotation: TypeId,
        help_text: impl Into<String> + Clone,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph<'sc>,
    ) -> CompileResult<'sc, (Self, TypeId)> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut engine: crate::type_engine::Engine = todo!("global type engine");

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
            match TYPE_ENGINE.lock().unwrap().unify_with_self(
                return_type,
                type_annotation,
                self_type,
                &implicit_return_span.unwrap_or(other.whole_block_span.clone()),
            ) {
                Ok(warning) => {
                    if let Some(warning) = warning {
                        warnings.push(CompileWarning {
                            warning_content: warning,
                            span: implicit_return_span.unwrap_or(other.whole_block_span.clone()),
                        });
                    }
                }
                Err(e) => {
                    errors.push(CompileError::TypeError(e));
                }
            };
            // The annotation will result in a cast, so set the return type accordingly.
        }
        /*
        if let Some(ref return_type) = return_type {
            let convertibility = return_type.is_convertible(
                &type_annotation,
                implicit_return_span.unwrap_or(other.whole_block_span.clone()),
                help_text,
            );
            match convertibility {
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
        */

        ok(
            (
                TypedCodeBlock {
                    contents: evaluated_contents,
                    whole_block_span: other.whole_block_span,
                },
                return_type.unwrap_or_else(|| engine.insert(TypeInfo::Unit)),
            ),
            warnings,
            errors,
        )
    }
}
