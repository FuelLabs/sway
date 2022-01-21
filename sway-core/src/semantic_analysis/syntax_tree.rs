use super::{
    node_dependencies, TypedAstNode, TypedAstNodeContent, TypedDeclaration,
    TypedFunctionDeclaration,
};

use crate::{
    build_config::BuildConfig,
    control_flow_analysis::ControlFlowGraph,
    error::*,
    parse_tree::Purity,
    semantic_analysis::{
        ast_node::Mode, retrieve_module, Namespace, NamespaceRef, TypeCheckArguments,
    },
    type_engine::*,
    AstNode, ParseTree,
};

use sway_types::{ident::Ident, span::Span};

use std::collections::{HashMap, HashSet};

/// Represents the different variants of the AST.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TreeType {
    Predicate,
    Script,
    Contract,
    Library { name: Ident },
}

#[derive(Debug, Clone)]
pub enum TypedParseTree {
    Script {
        main_function: TypedFunctionDeclaration,
        namespace: NamespaceRef,
        declarations: Vec<TypedDeclaration>,
        all_nodes: Vec<TypedAstNode>,
    },
    Predicate {
        main_function: TypedFunctionDeclaration,
        namespace: NamespaceRef,
        declarations: Vec<TypedDeclaration>,
        all_nodes: Vec<TypedAstNode>,
    },
    Contract {
        abi_entries: Vec<TypedFunctionDeclaration>,
        namespace: NamespaceRef,
        declarations: Vec<TypedDeclaration>,
        all_nodes: Vec<TypedAstNode>,
    },
    Library {
        namespace: NamespaceRef,
        all_nodes: Vec<TypedAstNode>,
    },
}

impl TypedParseTree {
    /// The `all_nodes` field in the AST variants is used to perform control flow and return flow
    /// analysis, while the direct copies of the declarations and main functions are used to create
    /// the ASM.
    pub(crate) fn all_nodes(&self) -> &[TypedAstNode] {
        use TypedParseTree::*;
        match self {
            Library { all_nodes, .. } => all_nodes,
            Script { all_nodes, .. } => all_nodes,
            Contract { all_nodes, .. } => all_nodes,
            Predicate { all_nodes, .. } => all_nodes,
        }
    }

    pub fn get_namespace_ref(self) -> NamespaceRef {
        use TypedParseTree::*;
        match self {
            Library { namespace, .. } => namespace,
            Script { namespace, .. } => namespace,
            Contract { namespace, .. } => namespace,
            Predicate { namespace, .. } => namespace,
        }
    }

    pub fn into_namespace(self) -> Namespace {
        use TypedParseTree::*;
        match self {
            Library { namespace, .. } => retrieve_module(namespace),
            Script { namespace, .. } => retrieve_module(namespace),
            Contract { namespace, .. } => retrieve_module(namespace),
            Predicate { namespace, .. } => retrieve_module(namespace),
        }
    }

    pub(crate) fn type_check(
        parsed: ParseTree,
        new_namespace: NamespaceRef,
        crate_namespace: NamespaceRef,
        tree_type: &TreeType,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
        dependency_graph: &mut HashMap<String, HashSet<String>>,
    ) -> CompileResult<Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let ordered_nodes = check!(
            node_dependencies::order_ast_nodes_by_dependency(parsed.root_nodes),
            return err(warnings, errors),
            warnings,
            errors
        );
        let typed_nodes = check!(
            TypedParseTree::type_check_nodes(
                ordered_nodes,
                new_namespace,
                crate_namespace,
                build_config,
                dead_code_graph,
                dependency_graph,
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

    fn type_check_nodes(
        nodes: Vec<AstNode>,
        namespace: NamespaceRef,
        crate_namespace: NamespaceRef,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
        dependency_graph: &mut HashMap<String, HashSet<String>>,
    ) -> CompileResult<Vec<TypedAstNode>> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let typed_nodes = nodes
            .into_iter()
            .map(|node| {
                TypedAstNode::type_check(TypeCheckArguments {
                    checkee: node,
                    namespace,
                    crate_namespace,
                    return_type_annotation: insert_type(TypeInfo::Unknown),
                    help_text: "",
                    self_type: insert_type(TypeInfo::Contract),
                    build_config,
                    dead_code_graph,
                    dependency_graph,
                    mode: Mode::NonAbi,
                    opts: Default::default(),
                })
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
        typed_tree_nodes: Vec<TypedAstNode>,
        span: Span,
        namespace: NamespaceRef,
        tree_type: &TreeType,
        warnings: Vec<CompileWarning>,
        mut errors: Vec<CompileError>,
    ) -> CompileResult<Self> {
        // Keep a copy of the nodes as they are.
        let all_nodes = typed_tree_nodes.clone();

        // Extract other interesting properties from the list.
        let mut mains = Vec::new();
        let mut declarations = Vec::new();
        let mut abi_entries = Vec::new();
        for node in typed_tree_nodes {
            match node.content {
                TypedAstNodeContent::Declaration(TypedDeclaration::FunctionDeclaration(func))
                    if func.name.as_str() == "main" =>
                {
                    mains.push(func)
                }
                // ABI entries are all functions declared in impl_traits on the contract type
                // itself.
                TypedAstNodeContent::Declaration(TypedDeclaration::ImplTrait {
                    methods,
                    type_implementing_for: TypeInfo::Contract,
                    ..
                }) => abi_entries.append(&mut methods.clone()),
                // XXX we're excluding the above ABI methods, is that OK?
                TypedAstNodeContent::Declaration(decl) => declarations.push(decl),
                _ => (),
            };
        }

        // impure functions are disallowed in non-contracts
        if *tree_type != TreeType::Contract {
            errors.append(&mut disallow_impure_functions(&declarations, &mains));
        }

        // Perform other validation based on the tree type.
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
                match look_up_type_id(main_func.return_type) {
                    TypeInfo::Boolean => (),
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
            TreeType::Library { .. } => TypedParseTree::Library {
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

fn disallow_impure_functions(
    declarations: &[TypedDeclaration],
    mains: &[TypedFunctionDeclaration],
) -> Vec<CompileError> {
    let fn_decls = declarations
        .iter()
        .filter_map(|decl| match decl {
            TypedDeclaration::FunctionDeclaration(decl) => Some(decl),
            _ => None,
        })
        .chain(mains);
    fn_decls
        .filter_map(|TypedFunctionDeclaration { purity, name, .. }| {
            if *purity == Purity::Impure {
                Some(CompileError::ImpureInNonContract {
                    span: name.span().clone(),
                })
            } else {
                None
            }
        })
        .collect()
}
