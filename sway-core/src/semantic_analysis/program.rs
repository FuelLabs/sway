use super::{
    TypedAstNode, TypedAstNodeContent, TypedDeclaration, TypedFunctionDeclaration, TypedImplTrait,
    TypedStorageDeclaration,
};
use crate::{
    error::*,
    parse_tree::{ParseProgram, Purity, TreeType},
    semantic_analysis::{
        namespace::{self, Namespace},
        TypeCheckContext, TypedModule,
    },
    type_engine::*,
    types::ToJsonAbi,
};
use fuels_types::JsonABI;
use sway_types::{span::Span, Ident, Spanned};

#[derive(Clone, Debug)]
pub struct TypedProgram {
    pub kind: TypedProgramKind,
    pub root: TypedModule,
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
        let ctx = TypeCheckContext::from_root(&mut namespace);
        let ParseProgram { root, kind } = parsed;
        let mod_span = root.tree.span.clone();
        let mod_res = TypedModule::type_check(ctx, root);
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
        let mut warnings = vec![];

        // Validate all submodules
        for (_, submodule) in &root.submodules {
            check!(
                Self::validate_root(
                    &submodule.module,
                    TreeType::Library {
                        name: submodule.library_name.clone(),
                    },
                    submodule.library_name.span().clone(),
                ),
                continue,
                warnings,
                errors
            );
        }

        let mut mains = Vec::new();
        let mut declarations = Vec::new();
        let mut abi_entries = Vec::new();
        let mut fn_declarations = std::collections::HashSet::new();
        for node in &root.all_nodes {
            match &node.content {
                TypedAstNodeContent::Declaration(TypedDeclaration::FunctionDeclaration(func))
                    if func.name.as_str() == "main" =>
                {
                    mains.push(func.clone())
                }
                // ABI entries are all functions declared in impl_traits on the contract type
                // itself.
                TypedAstNodeContent::Declaration(TypedDeclaration::ImplTrait(TypedImplTrait {
                    methods,
                    implementing_for_type_id,
                    ..
                })) if matches!(
                    look_up_type_id(*implementing_for_type_id),
                    TypeInfo::Contract
                ) =>
                {
                    abi_entries.extend(methods.clone())
                }
                // XXX we're excluding the above ABI methods, is that OK?
                TypedAstNodeContent::Declaration(decl) => {
                    // Variable and constant declarations don't need a duplicate check.
                    // Type declarations are checked elsewhere. That leaves functions.
                    if let TypedDeclaration::FunctionDeclaration(func) = &decl {
                        let name = func.name.clone();
                        if !fn_declarations.insert(name.clone()) {
                            errors.push(CompileError::MultipleDefinitionsOfFunction { name });
                        }
                    }
                    declarations.push(decl.clone())
                }
                _ => (),
            };
        }

        // Some checks that are specific to non-contracts
        if kind != TreeType::Contract {
            // impure functions are disallowed in non-contracts
            if !matches!(kind, TreeType::Library { .. }) {
                errors.extend(disallow_impure_functions(&declarations, &mains));
            }

            // `storage` declarations are not allowed in non-contracts
            let storage_decl = declarations
                .iter()
                .find(|decl| matches!(decl, TypedDeclaration::StorageDeclaration(_)));

            if let Some(TypedDeclaration::StorageDeclaration(TypedStorageDeclaration {
                span,
                ..
            })) = storage_decl
            {
                errors.push(CompileError::StorageDeclarationInNonContract {
                    program_kind: format!("{kind}"),
                    span: span.clone(),
                });
            }
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
                    errors.push(CompileError::MultipleDefinitionsOfFunction {
                        name: mains.last().unwrap().name.clone(),
                    });
                }
                let main_func = mains.remove(0);
                match look_up_type_id(main_func.return_type) {
                    TypeInfo::Boolean => (),
                    _ => errors.push(CompileError::PredicateMainDoesNotReturnBool(
                        main_func.span.clone(),
                    )),
                }
                TypedProgramKind::Predicate {
                    main_function: main_func,
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
                    errors.push(CompileError::MultipleDefinitionsOfFunction {
                        name: mains.last().unwrap().name.clone(),
                    });
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

impl ToJsonAbi for TypedProgramKind {
    type Output = JsonABI;

    // TODO: Update this to match behaviour described in the `compile` doc comment above.
    fn generate_json_abi(&self) -> Self::Output {
        match self {
            TypedProgramKind::Contract { abi_entries, .. } => {
                abi_entries.iter().map(|x| x.generate_json_abi()).collect()
            }
            TypedProgramKind::Script { main_function, .. } => {
                vec![main_function.generate_json_abi()]
            }
            _ => vec![],
        }
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
                Some(CompileError::ImpureInNonContract { span: name.span() })
            } else {
                None
            }
        })
        .collect()
}
