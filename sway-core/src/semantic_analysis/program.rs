use super::{
    storage_only_types, TypedAstNode, TypedAstNodeContent, TypedDeclaration,
    TypedFunctionDeclaration, TypedImplTrait, TypedStorageDeclaration,
};
use crate::{
    declaration_engine::declaration_engine::de_get_storage,
    error::*,
    metadata::MetadataManager,
    parse_tree::{ParseProgram, Purity, TreeType},
    semantic_analysis::{
        namespace::{self, Namespace},
        TypeCheckContext, TypedModule,
    },
    type_system::*,
};
use fuel_tx::StorageSlot;
use sway_ir::{Context, Module};
use sway_types::{
    span::Span, Ident, JsonABIProgram, JsonLoggedType, JsonTypeApplication, JsonTypeDeclaration,
    Spanned,
};

#[derive(Debug)]
pub struct TypedProgram {
    pub kind: TypedProgramKind,
    pub root: TypedModule,
    pub storage_slots: Vec<StorageSlot>,
    pub logged_types: Vec<TypeId>,
}

impl TypedProgram {
    /// Type-check the given parsed program to produce a typed program.
    ///
    /// The given `initial_namespace` acts as an initial state for each module within this program.
    /// It should contain a submodule for each library package dependency.
    pub fn type_check(
        parsed: &ParseProgram,
        initial_namespace: namespace::Module,
    ) -> CompileResult<Self> {
        let mut namespace = Namespace::init_root(initial_namespace);
        let ctx = TypeCheckContext::from_root(&mut namespace);
        let ParseProgram { root, kind } = parsed;
        let mod_span = root.tree.span.clone();
        let mod_res = TypedModule::type_check(ctx, root);
        mod_res.flat_map(|root| {
            let kind_res = Self::validate_root(&root, kind.clone(), mod_span);
            kind_res.map(|kind| Self {
                kind,
                root,
                storage_slots: vec![],
                logged_types: vec![],
            })
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

        for ast_n in &root.all_nodes {
            check!(
                storage_only_types::validate_decls_for_storage_only_types_in_ast(&ast_n.content),
                continue,
                warnings,
                errors
            );
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

            if let Some(TypedDeclaration::StorageDeclaration(decl_id)) = storage_decl {
                let TypedStorageDeclaration { span, .. } = check!(
                    CompileResult::from(de_get_storage(decl_id.clone(), &decl_id.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                errors.push(CompileError::StorageDeclarationInNonContract {
                    program_kind: format!("{kind}"),
                    span,
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
        // check if no ref mut arguments passed to a `main()` in a `script` or `predicate`.
        match &typed_program_kind {
            TypedProgramKind::Script { main_function, .. }
            | TypedProgramKind::Predicate { main_function, .. } => {
                for param in &main_function.parameters {
                    if param.is_reference && param.is_mutable {
                        errors.push(CompileError::RefMutableNotAllowedInMain {
                            param_name: param.name.clone(),
                        })
                    }
                }
            }
            _ => (),
        }
        ok(typed_program_kind, warnings, errors)
    }

    /// Ensures there are no unresolved types or types awaiting resolution in the AST.
    pub(crate) fn collect_types_metadata(&mut self) -> CompileResult<Vec<TypeMetadata>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        // Get all of the entry points for this tree type. For libraries, that's everything
        // public. For contracts, ABI entries. For scripts and predicates, any function named `main`.
        let metadata = match &self.kind {
            TypedProgramKind::Library { .. } => {
                let mut ret = vec![];
                for node in self.root.all_nodes.iter() {
                    let public = check!(
                        node.is_public(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    if public {
                        ret.append(&mut check!(
                            node.collect_types_metadata(),
                            return err(warnings, errors),
                            warnings,
                            errors
                        ));
                    }
                }
                ret
            }
            TypedProgramKind::Script { .. } => {
                let mut data = vec![];
                for node in self.root.all_nodes.iter() {
                    if node.is_main_function(TreeType::Script) {
                        data.append(&mut check!(
                            node.collect_types_metadata(),
                            return err(warnings, errors),
                            warnings,
                            errors
                        ));
                    }
                }
                data
            }
            TypedProgramKind::Predicate { .. } => {
                let mut data = vec![];
                for node in self.root.all_nodes.iter() {
                    if node.is_main_function(TreeType::Predicate) {
                        data.append(&mut check!(
                            node.collect_types_metadata(),
                            return err(warnings, errors),
                            warnings,
                            errors
                        ));
                    }
                }
                data
            }
            TypedProgramKind::Contract { abi_entries, .. } => {
                let mut data = vec![];
                for entry in abi_entries.iter() {
                    data.append(&mut check!(
                        TypedAstNode::from(entry).collect_types_metadata(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ));
                }
                data
            }
        };
        if errors.is_empty() {
            ok(metadata, warnings, errors)
        } else {
            err(warnings, errors)
        }
    }

    pub(crate) fn get_typed_program_with_initialized_storage_slots(
        &self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        module: Module,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        match &self.kind {
            TypedProgramKind::Contract { declarations, .. } => {
                let storage_decl = declarations
                    .iter()
                    .find(|decl| matches!(decl, TypedDeclaration::StorageDeclaration(_)));

                // Expecting at most a single storage declaration
                match storage_decl {
                    Some(TypedDeclaration::StorageDeclaration(decl_id)) => {
                        let decl = check!(
                            CompileResult::from(de_get_storage(decl_id.clone(), &decl_id.span())),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        let mut storage_slots = check!(
                            decl.get_initialized_storage_slots(context, md_mgr, module),
                            return err(warnings, errors),
                            warnings,
                            errors,
                        );
                        // Sort the slots to standardize the output. Not strictly required by the
                        // spec.
                        storage_slots.sort();
                        ok(
                            Self {
                                kind: self.kind.clone(),
                                root: self.root.clone(),
                                storage_slots,
                                logged_types: self.logged_types.clone(),
                            },
                            warnings,
                            errors,
                        )
                    }
                    _ => ok(
                        Self {
                            kind: self.kind.clone(),
                            root: self.root.clone(),
                            storage_slots: vec![],
                            logged_types: self.logged_types.clone(),
                        },
                        warnings,
                        errors,
                    ),
                }
            }
            _ => ok(
                Self {
                    kind: self.kind.clone(),
                    root: self.root.clone(),
                    storage_slots: vec![],
                    logged_types: self.logged_types.clone(),
                },
                warnings,
                errors,
            ),
        }
    }

    pub fn generate_json_abi_program(
        &self,
        types: &mut Vec<JsonTypeDeclaration>,
    ) -> JsonABIProgram {
        match &self.kind {
            TypedProgramKind::Contract { abi_entries, .. } => {
                let functions = abi_entries
                    .iter()
                    .map(|x| x.generate_json_abi_function(types))
                    .collect();
                let logged_types = self.generate_json_logged_types(types);
                JsonABIProgram {
                    types: types.to_vec(),
                    functions,
                    logged_types,
                }
            }
            TypedProgramKind::Script { main_function, .. }
            | TypedProgramKind::Predicate { main_function, .. } => {
                let functions = vec![main_function.generate_json_abi_function(types)];
                let logged_types = self.generate_json_logged_types(types);
                JsonABIProgram {
                    types: types.to_vec(),
                    functions,
                    logged_types,
                }
            }
            _ => JsonABIProgram {
                types: vec![],
                functions: vec![],
                logged_types: vec![],
            },
        }
    }

    fn generate_json_logged_types(
        &self,
        types: &mut Vec<JsonTypeDeclaration>,
    ) -> Vec<JsonLoggedType> {
        // A list of all `JsonTypeDeclaration`s needed for the logged types
        let logged_types = self
            .logged_types
            .iter()
            .map(|x| JsonTypeDeclaration {
                type_id: **x,
                type_field: x.get_json_type_str(*x),
                components: x.get_json_type_components(types, *x),
                type_parameters: x.get_json_type_parameters(types, *x),
            })
            .collect::<Vec<_>>();

        // Add the new types to `types`
        types.extend(logged_types);

        // Generate the JSON data for the logged types
        self.logged_types
            .iter()
            .map(|x| JsonLoggedType {
                log_id: **x,
                logged_type: JsonTypeApplication {
                    name: "".to_string(),
                    type_id: **x,
                    type_arguments: x.get_json_type_arguments(types, *x),
                },
            })
            .collect()
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
