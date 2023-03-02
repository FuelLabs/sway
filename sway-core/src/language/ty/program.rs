use crate::{
    decl_engine::*,
    error::*,
    fuel_prelude::fuel_tx::StorageSlot,
    language::{parsed, ty::*, Purity},
    semantic_analysis::storage_only_types,
    type_system::*,
    Engines,
};

use sway_error::error::CompileError;
use sway_types::*;

#[derive(Debug, Clone)]
pub struct TyProgram {
    pub kind: TyProgramKind,
    pub root: TyModule,
    pub declarations: Vec<TyDeclaration>,
    pub configurables: Vec<TyConstantDeclaration>,
    pub storage_slots: Vec<StorageSlot>,
    pub logged_types: Vec<(LogId, TypeId)>,
    pub messages_types: Vec<(MessageId, TypeId)>,
}

impl TyProgram {
    /// Validate the root module given the expected program kind.
    pub fn validate_root(
        engines: Engines<'_>,
        root: &TyModule,
        kind: parsed::TreeType,
        module_span: Span,
    ) -> CompileResult<(
        TyProgramKind,
        Vec<TyDeclaration>,
        Vec<TyConstantDeclaration>,
    )> {
        // Extract program-kind-specific properties from the root nodes.
        let mut errors = vec![];
        let mut warnings = vec![];

        let ty_engine = engines.te();
        let decl_engine = engines.de();

        // Validate all submodules
        let mut configurables = Vec::<TyConstantDeclaration>::new();
        for (_, submodule) in &root.submodules {
            check!(
                Self::validate_root(
                    engines,
                    &submodule.module,
                    parsed::TreeType::Library {
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
        let mut declarations = Vec::<TyDeclaration>::new();
        let mut abi_entries = Vec::new();
        let mut fn_declarations = std::collections::HashSet::new();
        for node in &root.all_nodes {
            match &node.content {
                TyAstNodeContent::Declaration(TyDeclaration::FunctionDeclaration {
                    name,
                    decl_id,
                    decl_span,
                }) => {
                    let func = decl_engine.get_function(decl_id);

                    if func.name.as_str() == "main" {
                        mains.push(func.clone());
                    }

                    if !fn_declarations.insert(func.name.clone()) {
                        errors.push(CompileError::MultipleDefinitionsOfFunction {
                            name: func.name.clone(),
                            span: func.name.span(),
                        });
                    }

                    declarations.push(TyDeclaration::FunctionDeclaration {
                        name: name.clone(),
                        decl_id: *decl_id,
                        decl_span: decl_span.clone(),
                    });
                }
                TyAstNodeContent::Declaration(TyDeclaration::ConstantDeclaration {
                    decl_id,
                    ..
                }) => {
                    let config_decl = decl_engine.get_constant(decl_id);
                    if config_decl.is_configurable {
                        configurables.push(config_decl);
                    }
                }
                // ABI entries are all functions declared in impl_traits on the contract type
                // itself, except for ABI supertraits, which do not expose their methods to
                // the user
                TyAstNodeContent::Declaration(TyDeclaration::ImplTrait { decl_id, .. }) => {
                    let TyImplTrait {
                        items,
                        implementing_for,

                        trait_decl_ref,
                        ..
                    } = decl_engine.get_impl_trait(decl_id);
                    if matches!(ty_engine.get(implementing_for.type_id), TypeInfo::Contract) {
                        // add methods to the ABI only if they come from an ABI implementation
                        // and not a (super)trait implementation for Contract
                        if let Some(trait_decl_ref) = trait_decl_ref {
                            if matches!(trait_decl_ref.id, InterfaceDeclId::Abi(_)) {
                                for item in items {
                                    match item {
                                        TyImplItem::Fn(method_ref) => {
                                            let method = decl_engine.get_function(&method_ref);
                                            abi_entries.push(method);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // XXX we're excluding the above ABI methods, is that OK?
                TyAstNodeContent::Declaration(decl) => {
                    declarations.push(decl.clone());
                }
                _ => {}
            };
        }

        for ast_n in &root.all_nodes {
            check!(
                storage_only_types::validate_decls_for_storage_only_types_in_ast(
                    engines,
                    &ast_n.content
                ),
                continue,
                warnings,
                errors
            );
        }

        // Some checks that are specific to non-contracts
        if kind != parsed::TreeType::Contract {
            // impure functions are disallowed in non-contracts
            if !matches!(kind, parsed::TreeType::Library { .. }) {
                errors.extend(disallow_impure_functions(
                    decl_engine,
                    &declarations,
                    &mains,
                ));
            }

            // `storage` declarations are not allowed in non-contracts
            let storage_decl = declarations
                .iter()
                .find(|decl| matches!(decl, TyDeclaration::StorageDeclaration { .. }));

            if let Some(TyDeclaration::StorageDeclaration { decl_span, .. }) = storage_decl {
                errors.push(CompileError::StorageDeclarationInNonContract {
                    program_kind: format!("{kind}"),
                    span: decl_span.clone(),
                });
            }
        }

        // Perform other validation based on the tree type.
        let typed_program_kind = match kind {
            parsed::TreeType::Contract => {
                // Types containing raw_ptr are not allowed in storage (e.g Vec)
                for decl in declarations.iter() {
                    if let TyDeclaration::StorageDeclaration {
                        decl_id,
                        decl_span: _,
                    } = decl
                    {
                        let storage_decl = decl_engine.get_storage(decl_id);
                        for field in storage_decl.fields.iter() {
                            let type_info = ty_engine.get(field.type_argument.type_id);
                            let type_info_str = engines.help_out(&type_info).to_string();
                            let raw_ptr_type = type_info
                                .extract_nested_types(ty_engine, &field.span)
                                .value
                                .and_then(|value| {
                                    value
                                        .into_iter()
                                        .find(|ty| matches!(ty, TypeInfo::RawUntypedPtr))
                                });
                            if raw_ptr_type.is_some() {
                                errors.push(CompileError::TypeNotAllowedInContractStorage {
                                    ty: type_info_str,
                                    span: field.span.clone(),
                                });
                            }
                        }
                    }
                }

                TyProgramKind::Contract { abi_entries }
            }
            parsed::TreeType::Library { name } => {
                if !configurables.is_empty() {
                    errors.push(CompileError::ConfigurableInLibrary {
                        span: configurables[0].name.span(),
                    });
                }
                TyProgramKind::Library { name }
            }
            parsed::TreeType::Predicate => {
                // A predicate must have a main function and that function must return a boolean.
                if mains.is_empty() {
                    errors.push(CompileError::NoPredicateMainFunction(module_span));
                    return err(vec![], errors);
                }
                if mains.len() > 1 {
                    errors.push(CompileError::MultipleDefinitionsOfFunction {
                        name: mains.last().unwrap().name.clone(),
                        span: mains.last().unwrap().name.span(),
                    });
                }
                let main_func = mains.remove(0);
                match ty_engine.get(main_func.return_type.type_id) {
                    TypeInfo::Boolean => (),
                    _ => errors.push(CompileError::PredicateMainDoesNotReturnBool(
                        main_func.span.clone(),
                    )),
                }
                TyProgramKind::Predicate {
                    main_function: main_func,
                }
            }
            parsed::TreeType::Script => {
                // A script must have exactly one main function.
                if mains.is_empty() {
                    errors.push(CompileError::NoScriptMainFunction(module_span));
                    return err(vec![], errors);
                }
                if mains.len() > 1 {
                    errors.push(CompileError::MultipleDefinitionsOfFunction {
                        name: mains.last().unwrap().name.clone(),
                        span: mains.last().unwrap().name.span(),
                    });
                }
                // A script must not return a `raw_ptr` or any type aggregating a `raw_slice`.
                // Directly returning a `raw_slice` is allowed, which will be just mapped to a RETD.
                // TODO: Allow returning nested `raw_slice`s when our spec supports encoding DSTs.
                let main_func = mains.remove(0);
                let main_return_type_info = ty_engine.get(main_func.return_type.type_id);
                let nested_types = check!(
                    main_return_type_info
                        .clone()
                        .extract_nested_types(ty_engine, &main_func.return_type.span),
                    vec![],
                    warnings,
                    errors
                );
                if nested_types
                    .iter()
                    .any(|ty| matches!(ty, TypeInfo::RawUntypedPtr))
                {
                    errors.push(CompileError::PointerReturnNotAllowedInMain {
                        span: main_func.return_type.span.clone(),
                    });
                }
                if !matches!(main_return_type_info, TypeInfo::RawUntypedSlice)
                    && nested_types
                        .iter()
                        .any(|ty| matches!(ty, TypeInfo::RawUntypedSlice))
                {
                    errors.push(CompileError::NestedSliceReturnNotAllowedInMain {
                        span: main_func.return_type.span.clone(),
                    });
                }
                TyProgramKind::Script {
                    main_function: main_func,
                }
            }
        };
        // check if no ref mut arguments passed to a `main()` in a `script` or `predicate`.
        match &typed_program_kind {
            TyProgramKind::Script { main_function, .. }
            | TyProgramKind::Predicate { main_function, .. } => {
                for param in &main_function.parameters {
                    if param.is_reference && param.is_mutable {
                        errors.push(CompileError::RefMutableNotAllowedInMain {
                            param_name: param.name.clone(),
                            span: param.name.span(),
                        })
                    }
                }
            }
            _ => (),
        }
        ok(
            (typed_program_kind, declarations, configurables),
            warnings,
            errors,
        )
    }

    /// All test function declarations within the program.
    pub fn test_fns<'a: 'b, 'b>(
        &'b self,
        decl_engine: &'a DeclEngine,
    ) -> impl '_ + Iterator<Item = (TyFunctionDeclaration, DeclRefFunction)> {
        self.root
            .submodules_recursive()
            .flat_map(|(_, submod)| submod.module.test_fns(decl_engine))
            .chain(self.root.test_fns(decl_engine))
    }
}

impl CollectTypesMetadata for TyProgram {
    /// Collect various type information such as unresolved types and types of logged data
    fn collect_types_metadata(
        &self,
        ctx: &mut CollectTypesMetadataContext,
    ) -> CompileResult<Vec<TypeMetadata>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let decl_engine = ctx.decl_engine;
        let mut metadata = vec![];

        // First, look into all entry points that are not unit tests.
        match &self.kind {
            // For scripts and predicates, collect metadata for all the types starting with
            // `main()` as the only entry point
            TyProgramKind::Script { main_function, .. }
            | TyProgramKind::Predicate { main_function, .. } => {
                metadata.append(&mut check!(
                    main_function.collect_types_metadata(ctx),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
            }
            // For contracts, collect metadata for all the types starting with each ABI method as
            // an entry point.
            TyProgramKind::Contract { abi_entries, .. } => {
                for entry in abi_entries.iter() {
                    metadata.append(&mut check!(
                        entry.collect_types_metadata(ctx),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ));
                }
            }
            // For libraries, collect metadata for all the types starting with each `pub` node as
            // an entry point. Also dig into all the submodules of a library because nodes in those
            // submodules can also be entry points.
            TyProgramKind::Library { .. } => {
                for module in std::iter::once(&self.root).chain(
                    self.root
                        .submodules_recursive()
                        .into_iter()
                        .map(|(_, submod)| &submod.module),
                ) {
                    for node in module.all_nodes.iter() {
                        let is_public = check!(
                            node.is_public(decl_engine),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        let is_generic_function = node.is_generic_function(decl_engine);
                        if is_public {
                            let node_metadata = check!(
                                node.collect_types_metadata(ctx),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            metadata.append(
                                &mut node_metadata
                                    .iter()
                                    .filter(|m| {
                                        // Generic functions are allowed to have unresolved types
                                        // so filter those
                                        !(is_generic_function
                                            && matches!(m, TypeMetadata::UnresolvedType(..)))
                                    })
                                    .cloned()
                                    .collect::<Vec<TypeMetadata>>(),
                            );
                        }
                    }
                }
            }
        }

        // Now consider unit tests: all unit test are considered entry points regardless of the
        // program type
        for module in std::iter::once(&self.root).chain(
            self.root
                .submodules_recursive()
                .into_iter()
                .map(|(_, submod)| &submod.module),
        ) {
            for node in module.all_nodes.iter() {
                if node.is_test_function(decl_engine) {
                    metadata.append(&mut check!(
                        node.collect_types_metadata(ctx),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ));
                }
            }
        }

        if errors.is_empty() {
            ok(metadata, warnings, errors)
        } else {
            err(warnings, errors)
        }
    }
}

#[derive(Clone, Debug)]
pub enum TyProgramKind {
    Contract {
        abi_entries: Vec<TyFunctionDeclaration>,
    },
    Library {
        name: Ident,
    },
    Predicate {
        main_function: TyFunctionDeclaration,
    },
    Script {
        main_function: TyFunctionDeclaration,
    },
}

impl TyProgramKind {
    /// The parse tree type associated with this program kind.
    pub fn tree_type(&self) -> parsed::TreeType {
        match self {
            TyProgramKind::Contract { .. } => parsed::TreeType::Contract,
            TyProgramKind::Library { name } => parsed::TreeType::Library { name: name.clone() },
            TyProgramKind::Predicate { .. } => parsed::TreeType::Predicate,
            TyProgramKind::Script { .. } => parsed::TreeType::Script,
        }
    }
    /// Used for project titles in `forc doc`.
    pub fn as_title_str(&self) -> &str {
        match self {
            TyProgramKind::Contract { .. } => "Contract",
            TyProgramKind::Library { .. } => "Library",
            TyProgramKind::Predicate { .. } => "Predicate",
            TyProgramKind::Script { .. } => "Script",
        }
    }
}

fn disallow_impure_functions(
    decl_engine: &DeclEngine,
    declarations: &[TyDeclaration],
    mains: &[TyFunctionDeclaration],
) -> Vec<CompileError> {
    let mut errs: Vec<CompileError> = vec![];
    let fn_decls = declarations
        .iter()
        .filter_map(|decl| match decl {
            TyDeclaration::FunctionDeclaration { decl_id, .. } => {
                Some(decl_engine.get_function(decl_id))
            }
            _ => None,
        })
        .chain(mains.to_owned());
    let mut err_purity = fn_decls
        .filter_map(|TyFunctionDeclaration { purity, name, .. }| {
            if purity != Purity::Pure {
                Some(CompileError::ImpureInNonContract { span: name.span() })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    errs.append(&mut err_purity);
    errs
}
