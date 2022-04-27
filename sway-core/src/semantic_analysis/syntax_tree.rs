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
        ast_node::Mode, namespace::arena::NamespaceWrapper, retrieve_module, Namespace,
        NamespaceRef, TypeCheckArguments,
    },
    type_engine::*,
    AstNode, ParseTree,
};
use sway_types::{ident::Ident, span::Span};

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

    /// Ensures there are no unresolved types or types awaiting resolution in the AST.
    pub(crate) fn finalize_types(&self) -> CompileResult<()> {
        use TypedParseTree::*;
        // Get all of the entry points for this tree type. For libraries, that's everything
        // public. For contracts, ABI entries. For scripts and predicates, any function named `main`.
        let errors: Vec<_> = match self {
            Library { all_nodes, .. } => all_nodes
                .iter()
                .filter(|x| x.is_public())
                .flat_map(UnresolvedTypeCheck::check_for_unresolved_types)
                .collect(),
            Script { all_nodes, .. } => all_nodes
                .iter()
                .filter(|x| x.is_main_function(TreeType::Script))
                .flat_map(UnresolvedTypeCheck::check_for_unresolved_types)
                .collect(),
            Predicate { all_nodes, .. } => all_nodes
                .iter()
                .filter(|x| x.is_main_function(TreeType::Predicate))
                .flat_map(UnresolvedTypeCheck::check_for_unresolved_types)
                .collect(),
            Contract { abi_entries, .. } => abi_entries
                .iter()
                .map(TypedAstNode::from)
                .flat_map(|x| x.check_for_unresolved_types())
                .collect(),
        };

        if errors.is_empty() {
            ok((), vec![], errors)
        } else {
            err(vec![], errors)
        }
    }

    pub(crate) fn type_check(
        parsed: ParseTree,
        new_namespace: NamespaceRef,
        crate_namespace: NamespaceRef,
        tree_type: &TreeType,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
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
                    help_text: Default::default(),
                    self_type: insert_type(TypeInfo::Contract),
                    build_config,
                    dead_code_graph,
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

        // Check that if trait B is a supertrait of trait A, and if A is implemented for type T,
        // then B is also implemented for type T
        errors.append(&mut check_supertraits(&all_nodes, &namespace));

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

/// Given a list of typed AST nodes and a namespace, check whether all supertrait constraints are
/// satisfied. We're basically checking the following condition:
///    if trait B is implemented for type T, then trait A_i is also implemented for type T for
///    every A_i such that A_i is a supertrait of B.
///
/// This nicely works for transitive supertraits as well.
///
fn check_supertraits(
    typed_tree_nodes: &[TypedAstNode],
    namespace: &NamespaceRef,
) -> Vec<CompileError> {
    let mut errors = vec![];
    for node in typed_tree_nodes {
        if let TypedAstNodeContent::Declaration(TypedDeclaration::ImplTrait {
            trait_name,
            span,
            type_implementing_for,
            ..
        }) = &node.content
        {
            if let CompileResult {
                value: Some(TypedDeclaration::TraitDeclaration(tr)),
                ..
            } = namespace.get_call_path(trait_name)
            {
                let supertraits = tr.supertraits;
                for supertrait in &supertraits {
                    if !typed_tree_nodes.iter().any(|search_node| {
                        if let TypedAstNodeContent::Declaration(TypedDeclaration::ImplTrait {
                            trait_name: search_node_trait_name,
                            type_implementing_for: search_node_type_implementing_for,
                            ..
                        }) = &search_node.content
                        {
                            if let (
                                CompileResult {
                                    value: Some(TypedDeclaration::TraitDeclaration(tr1)),
                                    ..
                                },
                                CompileResult {
                                    value: Some(TypedDeclaration::TraitDeclaration(tr2)),
                                    ..
                                },
                            ) = (
                                namespace.get_call_path(search_node_trait_name),
                                namespace.get_call_path(&supertrait.name),
                            ) {
                                return (tr1.name == tr2.name)
                                    && (type_implementing_for
                                        == search_node_type_implementing_for);
                            }
                        }
                        false
                    }) {
                        // The two errors below should really be a single error (and a "note"),
                        // but we don't have a way today to point to two separate locations in the
                        // user code with a single error.
                        errors.push(CompileError::SupertraitImplMissing {
                            supertrait_name: supertrait.name.to_string(),
                            type_name: type_implementing_for.friendly_type_str(),
                            span: span.clone(),
                        });
                        errors.push(CompileError::SupertraitImplRequired {
                            supertrait_name: supertrait.name.to_string(),
                            trait_name: tr.name.to_string(),
                            span: tr.name.span().clone(),
                        });
                    }
                }
            }
        }
    }
    errors
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
