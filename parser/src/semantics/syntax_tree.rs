use super::{TypedAstNode, TypedAstNodeContent, TypedDeclaration, TypedFunctionDeclaration};
use crate::error::*;
use crate::parse_tree::*;
use crate::types::{IntegerBits, TypeInfo};
use crate::{AstNode, AstNodeContent, CodeBlock, ParseTree, ReturnStatement, TraitFn};
use either::Either;
use pest::Span;
use std::collections::HashMap;

pub(crate) enum TreeType {
    Predicate,
    Script,
    Contract,
    Library,
}

#[derive(Debug)]
pub(crate) struct TypedParseTree<'sc> {
    root_nodes: Vec<TypedAstNode<'sc>>,
    pub(crate) namespace: HashMap<Ident<'sc>, TypedDeclaration<'sc>>,
    pub(crate) methods_namespace: HashMap<TypeInfo<'sc>, Vec<TypedFunctionDeclaration<'sc>>>,
}

impl<'sc> TypedParseTree<'sc> {
    pub(crate) fn type_check<'manifest>(
        parsed: ParseTree<'sc>,
        imported_namespace: &HashMap<
            &'manifest str,
            HashMap<Ident<'sc>, HashMap<Ident<'sc>, TypedDeclaration<'sc>>>,
        >,
        imported_method_namespace: &HashMap<
            &'manifest str,
            HashMap<Ident<'sc>, HashMap<TypeInfo<'sc>, Vec<TypedFunctionDeclaration<'sc>>>>,
        >,
        tree_type: TreeType,
    ) -> CompileResult<'sc, Self> {
        let mut global_namespace = Default::default();
        // a mapping from types to the methods that are available for them
        let mut methods_namespace = Default::default();
        let typed_tree = parsed
            .root_nodes
            .into_iter()
            .map(|node| {
                TypedAstNode::type_check(
                    node,
                    &mut global_namespace,
                    &mut methods_namespace,
                    imported_namespace,
                    imported_method_namespace,
                    None,
                    "",
                )
            })
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
                                return_type,
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
                namespace: global_namespace,
                methods_namespace,
            },
            warnings,
            errors,
        )
    }
}
