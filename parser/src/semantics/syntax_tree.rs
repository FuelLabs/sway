use super::{TypedAstNode, TypedAstNodeContent, TypedDeclaration, TypedFunctionDeclaration};
use crate::error::*;
use crate::semantics::Namespace;
use crate::types::TypeInfo;
use crate::ParseTree;

pub(crate) enum TreeType {
    Predicate,
    Script,
    Contract,
    Library,
}

#[derive(Debug)]
pub(crate) struct TypedParseTree<'sc> {
    root_nodes: Vec<TypedAstNode<'sc>>,
    pub(crate) namespace: Namespace<'sc>,
}

impl<'sc> TypedParseTree<'sc> {
    pub(crate) fn type_check<'manifest>(
        parsed: ParseTree<'sc>,
        initial_namespace: Namespace<'sc>,
        tree_type: TreeType,
    ) -> CompileResult<'sc, Self> {
        let mut initial_namespace = initial_namespace.clone();
        let typed_tree = parsed
            .root_nodes
            .into_iter()
            .map(|node| TypedAstNode::type_check(node, &mut initial_namespace, None, ""))
            .collect::<Vec<CompileResult<_>>>();

        let mut typed_tree_nodes = Vec::new();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        for res in typed_tree {
            match res {
                CompileResult::Ok {
                    value: node,
                    warnings: mut l_w,
                    errors: mut l_e,
                } => {
                    errors.append(&mut l_e);
                    warnings.append(&mut l_w);
                    typed_tree_nodes.push(node);
                }
                CompileResult::Err {
                    errors: mut l_e,
                    warnings: mut l_w,
                } => {
                    warnings.append(&mut l_w);
                    errors.append(&mut l_e);
                }
            }
        }
        // perform validation based on the tree type
        match tree_type {
            TreeType::Predicate => {
                // a predicate must have a main function and that function must return a boolean
                let main_func_vec = typed_tree_nodes
                    .iter()
                    .filter_map(|TypedAstNode { content, .. }| match content {
                        TypedAstNodeContent::Declaration(
                            TypedDeclaration::FunctionDeclaration(TypedFunctionDeclaration {
                                name,
                                return_type,
                                span,
                                ..
                            }),
                        ) => {
                            if name.primary_name == "main" {
                                Some((return_type, span))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();

                if main_func_vec.len() > 1 {
                    errors.push(CompileError::MultiplePredicateMainFunctions(
                        main_func_vec.last().unwrap().1.clone(),
                    ));
                } else if main_func_vec.is_empty() {
                    errors.push(CompileError::NoPredicateMainFunction(parsed.span));
                    return err(warnings, errors);
                }
                let main_func = main_func_vec[0];
                match main_func {
                    (TypeInfo::Boolean, _span) => (),
                    (_, span) => {
                        errors.push(CompileError::PredicateMainDoesNotReturnBool(span.clone()))
                    }
                }
            }
            TreeType::Script => {
                // a script must have exactly one main function
                let main_func_vec = typed_tree_nodes
                    .iter()
                    .filter_map(|TypedAstNode { content, .. }| match content {
                        TypedAstNodeContent::Declaration(
                            TypedDeclaration::FunctionDeclaration(TypedFunctionDeclaration {
                                name,
                                span,
                                ..
                            }),
                        ) => {
                            if name.primary_name == "main" {
                                Some(span)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();

                if main_func_vec.len() > 1 {
                    errors.push(CompileError::MultipleScriptMainFunctions(
                        main_func_vec.into_iter().last().unwrap().clone(),
                    ));
                } else if main_func_vec.is_empty() {
                    errors.push(CompileError::NoScriptMainFunction(parsed.span));
                    return err(warnings, errors);
                }
            }
            _ => (),
        }
        ok(
            TypedParseTree {
                root_nodes: typed_tree_nodes,
                namespace: initial_namespace,
            },
            warnings,
            errors,
        )
    }
}
