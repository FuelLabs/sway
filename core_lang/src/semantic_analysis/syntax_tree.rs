use super::{TypedAstNode, TypedAstNodeContent, TypedDeclaration, TypedFunctionDeclaration};
use crate::semantic_analysis::Namespace;
use crate::ParseTree;
use crate::{error::*, types::MaybeResolvedType};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TreeType {
    Predicate,
    Script,
    Contract,
    Library,
}

#[derive(Debug)]
pub(crate) enum TypedParseTree<'sc> {
    Script {
        main_function: TypedFunctionDeclaration<'sc>,
        namespace: Namespace<'sc>,
        declarations: Vec<TypedDeclaration<'sc>>,
        all_nodes: Vec<TypedAstNode<'sc>>,
    },
    Predicate {
        main_function: TypedFunctionDeclaration<'sc>,
        namespace: Namespace<'sc>,
        declarations: Vec<TypedDeclaration<'sc>>,
        all_nodes: Vec<TypedAstNode<'sc>>,
    },
    Contract {
        abi_entries: Vec<TypedFunctionDeclaration<'sc>>,
        namespace: Namespace<'sc>,
        declarations: Vec<TypedDeclaration<'sc>>,
        all_nodes: Vec<TypedAstNode<'sc>>,
    },
    Library {
        namespace: Namespace<'sc>,
        all_nodes: Vec<TypedAstNode<'sc>>,
    },
}

impl<'sc> TypedParseTree<'sc> {
    /// The `all_nodes` field in the AST variants is used to perform control flow and return flow
    /// analysis, while the direct copies of the declarations and main functions are used to create
    /// the ASM.
    pub(crate) fn all_nodes(&self) -> &[TypedAstNode<'sc>] {
        use TypedParseTree::*;
        match self {
            Library { all_nodes, .. } => all_nodes,
            Script { all_nodes, .. } => all_nodes,
            Contract { all_nodes, .. } => all_nodes,
            Predicate { all_nodes, .. } => all_nodes,
        }
    }
    pub(crate) fn namespace(&self) -> &Namespace<'sc> {
        use TypedParseTree::*;
        match self {
            Library { namespace, .. } => namespace,
            Script { namespace, .. } => namespace,
            Contract { namespace, .. } => namespace,
            Predicate { namespace, .. } => namespace,
        }
    }
    pub(crate) fn type_check(
        parsed: ParseTree<'sc>,
        initial_namespace: Namespace<'sc>,
        tree_type: TreeType,
    ) -> CompileResult<'sc, Self> {
        let mut initial_namespace = initial_namespace.clone();
        let typed_tree = parsed
            .root_nodes
            .into_iter()
            .map(|node| {
                TypedAstNode::type_check(
                    node,
                    &mut initial_namespace,
                    None,
                    "",
                    // TODO only allow impl traits on contract trees, do something else
                    // for other tree types
                    &MaybeResolvedType::Contract,
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
                let all_nodes = typed_tree_nodes.clone();
                let main_func_vec = typed_tree_nodes
                    .iter()
                    .filter_map(|TypedAstNode { content, .. }| match content {
                        TypedAstNodeContent::Declaration(
                            TypedDeclaration::FunctionDeclaration(func),
                        ) => {
                            if func.name.primary_name == "main" {
                                Some(func)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();

                if main_func_vec.len() > 1 {
                    errors.push(CompileError::MultiplePredicateMainFunctions(
                        main_func_vec.last().unwrap().span.clone(),
                    ));
                } else if main_func_vec.is_empty() {
                    errors.push(CompileError::NoPredicateMainFunction(parsed.span));
                    return err(warnings, errors);
                }
                let main_func = main_func_vec[0];
                match main_func.return_type {
                    MaybeResolvedType::Boolean => (),
                    _ => errors.push(CompileError::PredicateMainDoesNotReturnBool(
                        main_func.span.clone(),
                    )),
                }
                ok(
                    TypedParseTree::Predicate {
                        main_function: main_func.clone(),
                        all_nodes,
                        namespace: initial_namespace,
                        declarations: typed_tree_nodes
                            .into_iter()
                            .filter_map(|TypedAstNode { content, .. }| match content {
                                TypedAstNodeContent::Declaration(a) => Some(a),
                                _ => None,
                            })
                            .collect(),
                    },
                    warnings,
                    errors,
                )
            }
            TreeType::Script => {
                // a script must have exactly one main function
                let all_nodes = typed_tree_nodes.clone();
                let main_func_vec = typed_tree_nodes
                    .iter()
                    .filter_map(|TypedAstNode { content, .. }| match content {
                        TypedAstNodeContent::Declaration(
                            TypedDeclaration::FunctionDeclaration(func),
                        ) => {
                            if func.name.primary_name == "main" {
                                Some(func)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();

                if main_func_vec.len() > 1 {
                    errors.push(CompileError::MultipleScriptMainFunctions(
                        main_func_vec.last().unwrap().span.clone(),
                    ));
                } else if main_func_vec.is_empty() {
                    errors.push(CompileError::NoScriptMainFunction(parsed.span));
                    return err(warnings, errors);
                }

                let main_func = main_func_vec[0];

                ok(
                    TypedParseTree::Script {
                        main_function: main_func.clone(),
                        namespace: initial_namespace,
                        all_nodes,
                        declarations: typed_tree_nodes
                            .into_iter()
                            .filter_map(|TypedAstNode { content, .. }| match content {
                                TypedAstNodeContent::Declaration(a) => Some(a),
                                _ => None,
                            })
                            .collect(),
                    },
                    warnings,
                    errors,
                )
            }
            TreeType::Library => ok(
                TypedParseTree::Library {
                    all_nodes: typed_tree_nodes,
                    namespace: initial_namespace,
                },
                warnings,
                errors,
            ),
            TreeType::Contract => {
                // abi entries should be all public functions,
                // and all other declarations are not in the abi
                let mut abi_entries = vec![];
                let mut declarations = vec![];
                let all_nodes = typed_tree_nodes.clone();
                for node in typed_tree_nodes {
                    match node {
                        TypedAstNode {
                            content:
                                TypedAstNodeContent::Declaration(TypedDeclaration::FunctionDeclaration(
                                    a
                                    @
                                    TypedFunctionDeclaration {
                                        visibility: crate::Visibility::Public,
                                        ..
                                    },
                                )),
                            ..
                        } => abi_entries.push(a),
                        TypedAstNode {
                            content: TypedAstNodeContent::Declaration(a),
                            ..
                        } => declarations.push(a),
                        _ => (),
                    }
                }
                ok(
                    TypedParseTree::Contract {
                        abi_entries,
                        namespace: initial_namespace,
                        declarations,
                        all_nodes,
                    },
                    warnings,
                    errors,
                )
            }
        }
        /*
        ok(
            TypedParseTree {
                root_nodes: typed_tree_nodes,
                namespace: initial_namespace,
            },
            warnings,
            errors,
        )*/
    }
}
