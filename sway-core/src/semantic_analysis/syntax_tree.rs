use super::{
    node_dependencies, TypedAstNode, TypedAstNodeContent, TypedDeclaration,
    TypedFunctionDeclaration,
};

use crate::{
    error::*,
    parse_tree::{DepName, ParseModule, ParseProgram, ParseSubmodule, Purity, TreeType},
    semantic_analysis::{
        ast_node::Mode,
        namespace::{self, Namespace},
        TypeCheckArguments,
    },
    type_engine::*,
    AstNode,
};
use sway_types::{span::Span, Ident};

#[derive(Clone, Debug)]
pub struct TypedProgram {
    pub kind: TypedProgramKind,
    pub root: TypedModule,
}

#[derive(Clone, Debug)]
pub enum TypedProgramKind {
    Contract {
        abi_entries: Vec<TypedFunctionDeclaration>,
        declarations: Vec<TypedDeclaration>,
    },
    Library {
        name: Ident,
    },
    Predicate {
        main_function: TypedFunctionDeclaration,
        declarations: Vec<TypedDeclaration>,
    },
    Script {
        main_function: TypedFunctionDeclaration,
        declarations: Vec<TypedDeclaration>,
    },
}

#[derive(Clone, Debug)]
pub struct TypedModule {
    pub submodules: Vec<(DepName, TypedSubmodule)>,
    pub namespace: namespace::Module,
    pub all_nodes: Vec<TypedAstNode>,
}

#[derive(Clone, Debug)]
pub struct TypedSubmodule {
    pub library_name: Ident,
    pub module: TypedModule,
}

impl TypedProgram {
    /// Type-check the given parsed program to produce a typed program.
    ///
    /// The given `initial_namespace` acts as an initial state for each module within this program.
    /// It should contain a submodule for each library package dependency.
    pub fn type_check(
        parsed: ParseProgram,
        initial_namespace: namespace::Module,
    ) -> CompileResult<Self> {
        let mut namespace = Namespace::init_root(initial_namespace);
        let ParseProgram { root, kind } = parsed;
        let mod_span = root.tree.span.clone();
        let mod_res = TypedModule::type_check(root, &mut namespace);
        mod_res.flat_map(|root| {
            let kind_res = Self::validate_root(&root, kind, mod_span);
            kind_res.map(|kind| Self { kind, root })
        })
    }

    /// Validate the root module given the expected program kind.
    pub fn validate_root(
        root: &TypedModule,
        kind: TreeType,
        module_span: Span,
    ) -> CompileResult<TypedProgramKind> {
        // Extract program-kind-specific properties from the root nodes.
        let mut errors = vec![];
        let mut mains = Vec::new();
        let mut declarations = Vec::new();
        let mut abi_entries = Vec::new();
        for node in &root.all_nodes {
            match &node.content {
                TypedAstNodeContent::Declaration(TypedDeclaration::FunctionDeclaration(func))
                    if func.name.as_str() == "main" =>
                {
                    mains.push(func.clone())
                }
                // ABI entries are all functions declared in impl_traits on the contract type
                // itself.
                TypedAstNodeContent::Declaration(TypedDeclaration::ImplTrait {
                    methods,
                    type_implementing_for: TypeInfo::Contract,
                    ..
                }) => abi_entries.extend(methods.clone()),
                // XXX we're excluding the above ABI methods, is that OK?
                TypedAstNodeContent::Declaration(decl) => declarations.push(decl.clone()),
                _ => (),
            };
        }

        // impure functions are disallowed in non-contracts
        if kind != TreeType::Contract {
            errors.extend(disallow_impure_functions(&declarations, &mains));
        }

        // Perform other validation based on the tree type.
        let typed_program_kind = match kind {
            TreeType::Contract => TypedProgramKind::Contract {
                abi_entries,
                declarations,
            },
            TreeType::Library { name } => TypedProgramKind::Library { name },
            TreeType::Predicate => {
                // A predicate must have a main function and that function must return a boolean.
                if mains.is_empty() {
                    errors.push(CompileError::NoPredicateMainFunction(module_span));
                    return err(vec![], errors);
                }
                if mains.len() > 1 {
                    errors.push(CompileError::MultiplePredicateMainFunctions(
                        mains.last().unwrap().span.clone(),
                    ));
                }
                let main_func = mains.remove(0);
                match look_up_type_id(main_func.return_type) {
                    TypeInfo::Boolean => (),
                    _ => errors.push(CompileError::PredicateMainDoesNotReturnBool(
                        main_func.span.clone(),
                    )),
                }
                TypedProgramKind::Predicate {
                    main_function: main_func.clone(),
                    declarations,
                }
            }
            TreeType::Script => {
                // A script must have exactly one main function.
                if mains.is_empty() {
                    errors.push(CompileError::NoScriptMainFunction(module_span));
                    return err(vec![], errors);
                }
                if mains.len() > 1 {
                    errors.push(CompileError::MultipleScriptMainFunctions(
                        mains.last().unwrap().span.clone(),
                    ));
                }
                TypedProgramKind::Script {
                    main_function: mains.remove(0),
                    declarations,
                }
            }
        };

        ok(typed_program_kind, vec![], errors)
    }

    /// Ensures there are no unresolved types or types awaiting resolution in the AST.
    pub(crate) fn finalize_types(&self) -> CompileResult<()> {
        // Get all of the entry points for this tree type. For libraries, that's everything
        // public. For contracts, ABI entries. For scripts and predicates, any function named `main`.
        let errors: Vec<_> = match &self.kind {
            TypedProgramKind::Library { .. } => self
                .root
                .all_nodes
                .iter()
                .filter(|x| x.is_public())
                .flat_map(UnresolvedTypeCheck::check_for_unresolved_types)
                .collect(),
            TypedProgramKind::Script { .. } => self
                .root
                .all_nodes
                .iter()
                .filter(|x| x.is_main_function(TreeType::Script))
                .flat_map(UnresolvedTypeCheck::check_for_unresolved_types)
                .collect(),
            TypedProgramKind::Predicate { .. } => self
                .root
                .all_nodes
                .iter()
                .filter(|x| x.is_main_function(TreeType::Predicate))
                .flat_map(UnresolvedTypeCheck::check_for_unresolved_types)
                .collect(),
            TypedProgramKind::Contract { abi_entries, .. } => abi_entries
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
}

impl TypedModule {
    /// Type-check the given parsed module to produce a typed module.
    ///
    /// Recursively type-checks submodules first.
    pub fn type_check(parsed: ParseModule, namespace: &mut Namespace) -> CompileResult<Self> {
        let ParseModule { submodules, tree } = parsed;

        // Type-check submodules first in order of declaration.
        let mut submodules_res = ok(vec![], vec![], vec![]);
        for (name, submodule) in submodules {
            let submodule_res = TypedSubmodule::type_check(name.clone(), submodule, namespace);
            submodules_res = submodules_res.flat_map(|mut submodules| {
                submodule_res.map(|submodule| {
                    submodules.push((name, submodule));
                    submodules
                })
            });
        }

        // TODO: Ordering should be solved across all modules prior to the beginning of type-check.
        let ordered_nodes_res = node_dependencies::order_ast_nodes_by_dependency(tree.root_nodes);

        let typed_nodes_res = ordered_nodes_res
            .flat_map(|ordered_nodes| Self::type_check_nodes(ordered_nodes, namespace));

        let validated_nodes_res = typed_nodes_res.flat_map(|typed_nodes| {
            let errors = check_supertraits(&typed_nodes, namespace);
            ok(typed_nodes, vec![], errors)
        });

        submodules_res.flat_map(|submodules| {
            validated_nodes_res.map(|all_nodes| Self {
                submodules,
                namespace: namespace.module().clone(),
                all_nodes,
            })
        })
    }

    fn type_check_nodes(
        nodes: Vec<AstNode>,
        namespace: &mut Namespace,
    ) -> CompileResult<Vec<TypedAstNode>> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let typed_nodes = nodes
            .into_iter()
            .map(|node| {
                TypedAstNode::type_check(TypeCheckArguments {
                    checkee: node,
                    namespace,
                    return_type_annotation: insert_type(TypeInfo::Unknown),
                    help_text: Default::default(),
                    self_type: insert_type(TypeInfo::Contract),
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
}

impl TypedSubmodule {
    pub fn type_check(
        dep_name: DepName,
        submodule: ParseSubmodule,
        parent_namespace: &mut Namespace,
    ) -> CompileResult<Self> {
        let ParseSubmodule {
            library_name,
            module,
        } = submodule;
        let mut dep_namespace = parent_namespace.enter_submodule(dep_name);
        let module_res = TypedModule::type_check(module, &mut dep_namespace);
        module_res.map(|module| TypedSubmodule {
            library_name,
            module,
        })
    }
}

impl TypedProgramKind {
    /// The parse tree type associated with this program kind.
    pub fn tree_type(&self) -> TreeType {
        match self {
            TypedProgramKind::Contract { .. } => TreeType::Contract,
            TypedProgramKind::Library { name } => TreeType::Library { name: name.clone() },
            TypedProgramKind::Predicate { .. } => TreeType::Predicate,
            TypedProgramKind::Script { .. } => TreeType::Script,
        }
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
    namespace: &Namespace,
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
            } = namespace.resolve_call_path(trait_name)
            {
                for supertrait in &tr.supertraits {
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
                                namespace.resolve_call_path(search_node_trait_name),
                                namespace.resolve_call_path(&supertrait.name),
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
                            supertrait_name: supertrait.name.clone(),
                            type_name: type_implementing_for.friendly_type_str(),
                            span: span.clone(),
                        });
                        errors.push(CompileError::SupertraitImplRequired {
                            supertrait_name: supertrait.name.clone(),
                            trait_name: tr.name.clone(),
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
            if *purity != Purity::Pure {
                Some(CompileError::ImpureInNonContract {
                    span: name.span().clone(),
                })
            } else {
                None
            }
        })
        .collect()
}
