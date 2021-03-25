use super::*;
use crate::types::TypeInfo;
use crate::CodeBlock;

#[derive(Clone, Debug)]
pub(crate) struct TypedCodeBlock<'sc> {
    pub(crate) contents: Vec<TypedAstNode<'sc>>,
}

impl<'sc> TypedCodeBlock<'sc> {
    pub(crate) fn type_check<'manifest>(
        other: CodeBlock<'sc>,
        namespace: &Namespace<'sc>,
        // this is for the return or implicit return
        type_annotation: Option<TypeInfo<'sc>>,
        help_text: impl Into<String> + Clone,
    ) -> CompileResult<'sc, (Self, TypeInfo<'sc>)> {
        // TODO implicit returns from blocks
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut evaluated_contents = Vec::new();
        // mutable clone, because the interior of a code block can not change the surrounding
        // namespace
        let mut local_namespace = namespace.clone();
        let last_node = other
            .contents
            .last()
            .expect("empty code block? TODO check if this is handled earlier")
            .clone();
        for node in &other.contents[0..other.contents.len() - 1] {
            match TypedAstNode::type_check(node.clone(), &mut local_namespace, None, "") {
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
        // now, handle the final line with the type annotation.
        let res = match TypedAstNode::type_check(
            last_node.clone(),
            &mut local_namespace,
            type_annotation.clone(),
            help_text.clone(),
        ) {
            CompileResult::Ok {
                value,
                errors: mut l_e,
                warnings: mut l_w,
            } => {
                warnings.append(&mut l_w);
                errors.append(&mut l_e);
                value
            }
            CompileResult::Err {
                errors: mut l_e,
                warnings: mut l_w,
            } => {
                warnings.append(&mut l_w);
                errors.append(&mut l_e);
                TypedAstNode {
                    content: ERROR_RECOVERY_NODE_CONTENT.clone(),
                    span: last_node.span,
                }
            }
        };
        evaluated_contents.push(res.clone());
        if let Some(type_annotation) = type_annotation {
            let convertability = res.type_info().is_convertable(
                type_annotation.clone(),
                res.span.clone(),
                help_text,
            );
            match convertability {
                Ok(warning) => {
                    if let Some(warning) = warning {
                        warnings.push(CompileWarning {
                            warning_content: warning,
                            span: res.span.clone(),
                        });
                    }
                }
                Err(err) => {
                    errors.push(err.into());
                }
            }
        }

        ok(
            (
                TypedCodeBlock {
                    contents: evaluated_contents,
                },
                res.type_info(),
            ),
            warnings,
            errors,
        )
    }
}
