use super::*;
use crate::types::ResolvedType;
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
        type_annotation: Option<ResolvedType<'sc>>,
        help_text: impl Into<String> + Clone,
    ) -> CompileResult<'sc, (Self, Option<ResolvedType<'sc>>)> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut evaluated_contents = Vec::new();
        // mutable clone, because the interior of a code block can not change the surrounding
        // namespace
        let mut local_namespace = namespace.clone();
        // use this span for an error later
        let implicit_return_span = other
            .contents
            .iter()
            .find_map(|x| match &x.content {
                AstNodeContent::ImplicitReturnExpression(expr) => Some(Some(expr.span())),
                _ => None,
            })
            .unwrap_or(None);
        for node in &other.contents {
            match TypedAstNode::type_check(
                node.clone(),
                &mut local_namespace,
                type_annotation.clone(),
                help_text.clone(),
            ) {
                CompileResult::Ok {
                    value,
                    warnings: mut l_w,
                    errors: mut l_e,
                } => {
                    warnings.append(&mut l_w);
                    errors.append(&mut l_e);
                    evaluated_contents.push(value);
                }
                CompileResult::Err {
                    errors: mut l_e,
                    warnings: mut l_w,
                } => {
                    warnings.append(&mut l_w);
                    errors.append(&mut l_e);
                }
            };
        }
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
                    implicit_return_span.unwrap_or(other.whole_block_span.clone()),
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
