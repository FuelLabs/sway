use super::{TypedAstNode, TypedAstNodeContent, TypedDeclaration, TypedFunctionDeclaration};
use crate::build_config::BuildConfig;
use crate::control_flow_analysis::ControlFlowGraph;
use crate::semantic_analysis::Namespace;
use crate::{
    error::*,
    types::{MaybeResolvedType, ResolvedType},
};
use crate::{AstNode, ParseTree};
use std::collections::VecDeque;

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
    ) -> CompileResult<'sc, Self> {
        let mut initial_namespace = initial_namespace.clone();

        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let typed_nodes = check!(
            TypedParseTree::get_typed_nodes(
                parsed.root_nodes,
                &mut initial_namespace,
                build_config,
                dead_code_graph
            ),
            return err(warnings, errors),
            warnings,
            errors
        );

        TypedParseTree::validate_typed_nodes(
            parsed.span,
            initial_namespace,
            tree_type,
            warnings,
            errors,
            typed_nodes,
        )
    }

    fn get_typed_nodes(
        root_nodes: Vec<AstNode<'sc>>,
        initial_namespace: &mut Namespace<'sc>,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph<'sc>,
    ) -> CompileResult<'sc, Vec<TypedAstNode<'sc>>> {
        let mut successful_nodes = vec![];
        let mut next_pass_nodes: VecDeque<_> = root_nodes.into_iter().collect();
        let mut num_failed_nodes = next_pass_nodes.len();

        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let mut is_first_pass = true;
        while num_failed_nodes > 0 {
            let nodes = next_pass_nodes
                .clone()
                .into_iter()
                .map(|node| {
                    (
                        node.clone(),
                        TypedAstNode::type_check(
                            node,
                            initial_namespace,
                            None,
                            "",
                            // TODO only allow impl traits on contract trees, do something else
                            // for other tree types
                            &MaybeResolvedType::Resolved(ResolvedType::Contract),
                            build_config,
                            dead_code_graph,
                        ),
                    )
                })
                .collect::<Vec<(_, CompileResult<_>)>>();
            next_pass_nodes = Default::default();

            // If we hit the internal "non-decreasing error nodes" error, this helps
            // show what went wrong right beforehand.
            let mut errors_from_this_pass = vec![];
            for (node, mut res) in nodes.clone() {
                if res.value.is_none() {
                    errors_from_this_pass.append(&mut res.errors);
                    next_pass_nodes.push_front(node);
                } else {
                    if res.errors.is_empty() {
                        successful_nodes.push(res);
                    } else {
                        errors_from_this_pass.append(&mut res.errors);
                        next_pass_nodes.push_front(node);
                    }
                }
            }
            // If we did not solve any issues, i.e. the same number of nodes failed,
            // then this is a genuine error and so we break.
            if next_pass_nodes.len() == num_failed_nodes && !is_first_pass {
                for (_, mut failed_node_res) in nodes {
                    warnings.append(&mut failed_node_res.warnings);
                    errors.append(&mut failed_node_res.errors);
                }
                break;
            }
            is_first_pass = false;
            // if the amount of nodes with errors is going up, then bail.
            if next_pass_nodes.len() > num_failed_nodes {
                errors.append(&mut errors_from_this_pass);
                return err(warnings, errors);
            }
            num_failed_nodes = next_pass_nodes.len();
        }

        // gather nodes, warnings and errors together
        ok(
            successful_nodes
                .into_iter()
                .filter_map(|res| res.ok(&mut warnings, &mut errors))
                .collect::<Vec<TypedAstNode<'sc>>>(),
            warnings,
            errors,
        )
    }

    fn validate_typed_nodes(
        span: pest::Span<'sc>,
        namespace: Namespace<'sc>,
        tree_type: TreeType,
        warnings: Vec<CompileWarning<'sc>>,
        mut errors: Vec<CompileError<'sc>>,
        typed_tree_nodes: Vec<TypedAstNode<'sc>>,
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
                    declarations: declarations,
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
