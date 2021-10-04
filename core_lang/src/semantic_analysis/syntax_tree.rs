use super::node_dependencies;
use super::{TypedAstNode, TypedAstNodeContent, TypedDeclaration, TypedFunctionDeclaration};
use crate::build_config::BuildConfig;
use crate::control_flow_analysis::ControlFlowGraph;
use crate::semantic_analysis::Namespace;
use crate::span::Span;
use crate::{
    error::*,
    types::{MaybeResolvedType, ResolvedType},
};
use crate::{AstNode, AstNodeContent, ParseTree};
use std::collections::{HashMap, HashSet};

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
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph<'sc>,
        dependency_graph: &mut HashMap<String, HashSet<String>>,
    ) -> CompileResult<'sc, Self> {
        let mut new_namespace = initial_namespace.clone();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let ordered_nodes = check!(
            node_dependencies::order_ast_nodes_by_dependency(parsed.root_nodes),
            return err(warnings, errors),
            warnings,
            errors
        );

        check!(
            TypedParseTree::check_for_infinite_dependencies(
                build_config.file_name.clone().to_str().unwrap().to_string(),
                &ordered_nodes,
                dependency_graph,
            ),
            return err(warnings, errors),
            warnings,
            errors
        );

        let typed_nodes = check!(
            TypedParseTree::type_check_nodes(
                ordered_nodes,
                &mut new_namespace,
                build_config,
                dead_code_graph,
                dependency_graph
            ),
            return err(warnings, errors),
            warnings,
            errors
        );

        TypedParseTree::validate_typed_nodes(
            typed_nodes,
            parsed.span,
            new_namespace,
            tree_type,
            warnings,
            errors,
        )
    }

    fn check_for_infinite_dependencies(
        file_path: String,
        ordered_nodes: &Vec<AstNode<'sc>>,
        dependency_graph: &mut HashMap<String, HashSet<String>>,
    ) -> CompileResult<'sc, ()> {
        let warnings = vec![];
        let mut errors = vec![];

        for node in ordered_nodes.iter() {
            match &node.content {
                AstNodeContent::IncludeStatement(include_statement) => {
                    let file_path2 = include_statement.file_path;
                    let orig_file_path = &(file_path.clone());
                    if let Some(value) = dependency_graph.get_mut(&file_path) {
                        if value.contains(&file_path2.to_string()) {
                            errors.push(CompileError::InfiniteDependencies {
                                file_path: file_path,
                                span: node.span.clone(),
                            });
                            return err(warnings, errors);
                        } else {
                            value.insert(file_path2.to_string());
                        }
                    } else {
                        let mut set = HashSet::new();
                        set.insert(file_path2.to_string());
                        dependency_graph.insert(orig_file_path.clone(), set);
                    }
                }
                _ => {}
            }
        }

        ok((), warnings, errors)
    }

    fn type_check_nodes(
        nodes: Vec<AstNode<'sc>>,
        namespace: &mut Namespace<'sc>,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph<'sc>,
        dependency_graph: &mut HashMap<String, HashSet<String>>,
    ) -> CompileResult<'sc, Vec<TypedAstNode<'sc>>> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let typed_nodes = nodes
            .into_iter()
            .map(|node| {
                TypedAstNode::type_check(
                    node.clone(),
                    namespace,
                    None,
                    "",
                    // TODO only allow impl traits on contract trees, do something else
                    // for other tree types
                    &MaybeResolvedType::Resolved(ResolvedType::Contract),
                    build_config,
                    dead_code_graph,
                    dependency_graph,
                )
            })
            .filter_map(|res| res.ok(&mut warnings, &mut errors))
            .collect();

        if !errors.is_empty() {
            err(warnings, errors)
        } else {
            ok(typed_nodes, warnings, errors)
        }
    }

    fn validate_typed_nodes(
        typed_tree_nodes: Vec<TypedAstNode<'sc>>,
        span: Span<'sc>,
        namespace: Namespace<'sc>,
        tree_type: TreeType,
        warnings: Vec<CompileWarning<'sc>>,
        mut errors: Vec<CompileError<'sc>>,
    ) -> CompileResult<'sc, Self> {
        // Keep a copy of the nodes as they are.
        let all_nodes = typed_tree_nodes.clone();

        // Extract other interesting properties from the list.
        let mut mains = Vec::new();
        let mut declarations = Vec::new();
        let mut abi_entries = Vec::new();
        for node in typed_tree_nodes {
            match node.content {
                TypedAstNodeContent::Declaration(TypedDeclaration::FunctionDeclaration(func))
                    if func.name.primary_name == "main" =>
                {
                    mains.push(func)
                }
                // ABI entries are all functions declared in impl_traits on the contract type
                // itself.
                TypedAstNodeContent::Declaration(TypedDeclaration::ImplTrait {
                    methods,
                    type_implementing_for: MaybeResolvedType::Resolved(ResolvedType::Contract),
                    ..
                }) => abi_entries.append(&mut methods.clone()),
                // XXX we're excluding the above ABI methods, is that OK?
                TypedAstNodeContent::Declaration(decl) => declarations.push(decl),
                _ => (),
            };
        }

        // Perform validation based on the tree type.
        let typed_parse_tree = match tree_type {
            TreeType::Predicate => {
                // A predicate must have a main function and that function must return a boolean.
                if mains.is_empty() {
                    errors.push(CompileError::NoPredicateMainFunction(span));
                    return err(warnings, errors);
                }
                if mains.len() > 1 {
                    errors.push(CompileError::MultiplePredicateMainFunctions(
                        mains.last().unwrap().span.clone(),
                    ));
                }
                let main_func = &mains[0];
                match main_func.return_type {
                    MaybeResolvedType::Resolved(ResolvedType::Boolean) => (),
                    _ => errors.push(CompileError::PredicateMainDoesNotReturnBool(
                        main_func.span.clone(),
                    )),
                }
                TypedParseTree::Predicate {
                    main_function: main_func.clone(),
                    all_nodes,
                    namespace,
                    declarations,
                }
            }
            TreeType::Script => {
                // A script must have exactly one main function.
                if mains.is_empty() {
                    errors.push(CompileError::NoScriptMainFunction(span));
                    return err(warnings, errors);
                }
                if mains.len() > 1 {
                    errors.push(CompileError::MultipleScriptMainFunctions(
                        mains.last().unwrap().span.clone(),
                    ));
                }
                TypedParseTree::Script {
                    main_function: mains[0].clone(),
                    all_nodes,
                    namespace,
                    declarations,
                }
            }
            TreeType::Library => TypedParseTree::Library {
                all_nodes,
                namespace,
            },
            TreeType::Contract => TypedParseTree::Contract {
                abi_entries,
                namespace,
                declarations,
                all_nodes,
            },
        };

        ok(typed_parse_tree, warnings, errors)
    }
}
