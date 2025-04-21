use std::sync::Arc;

use crate::{
    decl_engine::*,
    fuel_prelude::fuel_tx::StorageSlot,
    language::{parsed, ty::*, Purity, Visibility},
    namespace::{check_impls_for_overlap, check_orphan_rules_for_impls, TraitMap},
    semantic_analysis::namespace,
    transform::AllowDeprecatedState,
    type_system::*,
    types::*,
    Engines,
};

use sway_error::{
    error::{CompileError, TypeNotAllowedReason},
    handler::{ErrorEmitted, Handler},
};
use sway_features::ExperimentalFeatures;
use sway_types::*;

#[derive(Debug, Clone)]
pub struct TyProgram {
    pub kind: TyProgramKind,
    pub root_module: TyModule,
    pub namespace: namespace::Namespace,
    pub declarations: Vec<TyDecl>,
    pub configurables: Vec<TyConfigurableDecl>,
    pub storage_slots: Vec<StorageSlot>,
    pub logged_types: Vec<(LogId, TypeId)>,
    pub messages_types: Vec<(MessageId, TypeId)>,
}

fn get_type_not_allowed_error(
    engines: &Engines,
    type_id: TypeId,
    spanned: &impl Spanned,
    f: impl Fn(&TypeInfo) -> Option<TypeNotAllowedReason>,
) -> Option<CompileError> {
    let types = type_id.extract_any_including_self(engines, &|t| f(t).is_some(), vec![], 0);

    let (id, _) = types.into_iter().next()?;
    let t = engines.te().get(id);

    Some(CompileError::TypeNotAllowed {
        reason: f(&t)?,
        span: spanned.span(),
    })
}

fn check_no_ref_main(engines: &Engines, handler: &Handler, main_function: &DeclId<TyFunctionDecl>) {
    let main_function = engines.de().get_function(main_function);
    for param in main_function.parameters.iter() {
        if param.is_reference && param.is_mutable {
            handler.emit_err(CompileError::RefMutableNotAllowedInMain {
                param_name: param.name.clone(),
                span: param.name.span(),
            });
        }
    }
}

impl TyProgram {
    pub fn validate_coherence(
        handler: &Handler,
        engines: &Engines,
        root: &TyModule,
        root_namespace: &mut namespace::Namespace,
    ) -> Result<(), ErrorEmitted> {
        // check orphan rules for all traits
        check_orphan_rules_for_impls(handler, engines, root_namespace.current_package_ref())?;

        // check trait overlap
        let mut unified_trait_map = root_namespace
            .current_package_ref()
            .root_module()
            .root_lexical_scope()
            .items
            .implemented_traits
            .clone();

        Self::validate_coherence_overlap(
            handler,
            engines,
            root,
            root_namespace,
            &mut unified_trait_map,
        )?;

        Ok(())
    }

    pub fn validate_coherence_overlap(
        handler: &Handler,
        engines: &Engines,
        module: &TyModule,
        root_namespace: &mut namespace::Namespace,
        unified_trait_map: &mut TraitMap,
    ) -> Result<(), ErrorEmitted> {
        let other_trait_map = unified_trait_map.clone();
        check_impls_for_overlap(unified_trait_map, handler, other_trait_map, engines)?;

        for (submod_name, submodule) in module.submodules.iter() {
            root_namespace.push_submodule(
                handler,
                engines,
                submod_name.clone(),
                Visibility::Public,
                submodule.mod_name_span.clone(),
                false,
            )?;

            Self::validate_coherence_overlap(
                handler,
                engines,
                &submodule.module,
                root_namespace,
                unified_trait_map,
            )?;

            root_namespace.pop_submodule();
        }

        Ok(())
    }

    /// Validate the root module given the expected program kind.
    pub fn validate_root(
        handler: &Handler,
        engines: &Engines,
        root: &TyModule,
        kind: parsed::TreeType,
        package_name: &str,
        experimental: ExperimentalFeatures,
    ) -> Result<(TyProgramKind, Vec<TyDecl>, Vec<TyConfigurableDecl>), ErrorEmitted> {
        // Extract program-kind-specific properties from the root nodes.

        let ty_engine = engines.te();
        let decl_engine = engines.de();

        // Validate all submodules
        let mut configurables = vec![];
        for (_, submodule) in &root.submodules {
            let _ = Self::validate_root(
                handler,
                engines,
                &submodule.module,
                parsed::TreeType::Library,
                package_name,
                experimental,
            );
        }

        let mut entries = Vec::new();
        let mut mains = Vec::new();
        let mut declarations = Vec::<TyDecl>::new();
        let mut abi_entries = Vec::new();
        let mut fn_declarations = std::collections::HashSet::new();

        for node in &root.all_nodes {
            match &node.content {
                TyAstNodeContent::Declaration(TyDecl::FunctionDecl(FunctionDecl { decl_id })) => {
                    let func = decl_engine.get_function(decl_id);

                    match func.kind {
                        TyFunctionDeclKind::Main => mains.push(*decl_id),
                        TyFunctionDeclKind::Entry => entries.push(*decl_id),
                        _ => {}
                    }

                    if !fn_declarations.insert(func.name.clone()) {
                        handler.emit_err(CompileError::MultipleDefinitionsOfFunction {
                            name: func.name.clone(),
                            span: func.name.span(),
                        });
                    }

                    declarations.push(TyDecl::FunctionDecl(FunctionDecl { decl_id: *decl_id }));
                }
                TyAstNodeContent::Declaration(TyDecl::ConfigurableDecl(ConfigurableDecl {
                    decl_id,
                    ..
                })) => {
                    let decl = (*decl_engine.get_configurable(decl_id)).clone();
                    configurables.push(decl);
                }
                // ABI entries are all functions declared in impl_traits on the contract type
                // itself, except for ABI supertraits, which do not expose their methods to
                // the user
                TyAstNodeContent::Declaration(TyDecl::ImplSelfOrTrait(ImplSelfOrTrait {
                    decl_id,
                    ..
                })) => {
                    let impl_trait_decl = decl_engine.get_impl_self_or_trait(decl_id);
                    let TyImplSelfOrTrait {
                        items,
                        implementing_for,
                        trait_decl_ref,
                        ..
                    } = &*impl_trait_decl;
                    if matches!(
                        &*ty_engine.get(implementing_for.type_id()),
                        TypeInfo::Contract
                    ) {
                        // add methods to the ABI only if they come from an ABI implementation
                        // and not a (super)trait implementation for Contract
                        if let Some(trait_decl_ref) = trait_decl_ref {
                            if matches!(*trait_decl_ref.id(), InterfaceDeclId::Abi(_)) {
                                for item in items {
                                    match item {
                                        TyImplItem::Fn(method_ref) => {
                                            abi_entries.push(*method_ref.id());
                                        }
                                        TyImplItem::Constant(const_ref) => {
                                            declarations.push(TyDecl::ConstantDecl(ConstantDecl {
                                                decl_id: *const_ref.id(),
                                            }));
                                        }
                                        TyImplItem::Type(type_ref) => {
                                            declarations.push(TyDecl::TraitTypeDecl(
                                                TraitTypeDecl {
                                                    decl_id: *type_ref.id(),
                                                },
                                            ));
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

        // Some checks that are specific to non-contracts
        if kind != parsed::TreeType::Contract {
            // impure functions are disallowed in non-contracts
            if !matches!(kind, parsed::TreeType::Library { .. }) {
                for err in disallow_impure_functions(decl_engine, &declarations, &entries) {
                    handler.emit_err(err);
                }
            }

            // `storage` declarations are not allowed in non-contracts
            let storage_decl = declarations
                .iter()
                .find(|decl| matches!(decl, TyDecl::StorageDecl { .. }));

            if let Some(TyDecl::StorageDecl(StorageDecl { decl_id })) = storage_decl {
                handler.emit_err(CompileError::StorageDeclarationInNonContract {
                    program_kind: format!("{kind}"),
                    span: engines.de().get(decl_id).span.clone(),
                });
            }
        }

        // Perform other validation based on the tree type.
        let typed_program_kind = match kind {
            parsed::TreeType::Contract => {
                // Types containing raw_ptr are not allowed in storage (e.g Vec)
                for decl in declarations.iter() {
                    if let TyDecl::StorageDecl(StorageDecl { decl_id }) = decl {
                        let storage_decl = decl_engine.get_storage(decl_id);
                        for field in storage_decl.fields.iter() {
                            if let Some(error) = get_type_not_allowed_error(
                                engines,
                                field.type_argument.type_id(),
                                &field.type_argument,
                                |t| match t {
                                    TypeInfo::StringSlice => {
                                        Some(TypeNotAllowedReason::StringSliceInConfigurables)
                                    }
                                    TypeInfo::RawUntypedPtr => Some(
                                        TypeNotAllowedReason::TypeNotAllowedInContractStorage {
                                            ty: engines.help_out(t).to_string(),
                                        },
                                    ),
                                    _ => None,
                                },
                            ) {
                                handler.emit_err(error);
                            }
                        }
                    }
                }

                TyProgramKind::Contract {
                    entry_function: if experimental.new_encoding {
                        if entries.len() != 1 {
                            return Err(handler.emit_err(CompileError::CouldNotGenerateEntry {
                                span: Span::dummy(),
                            }));
                        }
                        Some(entries[0])
                    } else {
                        None
                    },
                    abi_entries,
                }
            }
            parsed::TreeType::Library => {
                if !configurables.is_empty() {
                    handler.emit_err(CompileError::ConfigurableInLibrary {
                        span: configurables[0].call_path.suffix.span(),
                    });
                }
                TyProgramKind::Library {
                    name: package_name.to_string(),
                }
            }
            parsed::TreeType::Predicate => {
                if mains.is_empty() {
                    return Err(
                        handler.emit_err(CompileError::NoPredicateMainFunction(root.span.clone()))
                    );
                }

                if mains.len() > 1 {
                    let mut last_error = None;
                    for m in mains.iter().skip(1) {
                        let mains_last = decl_engine.get_function(m);
                        last_error = Some(handler.emit_err(
                            CompileError::MultipleDefinitionsOfFunction {
                                name: mains_last.name.clone(),
                                span: mains_last.name.span(),
                            },
                        ));
                    }
                    return Err(last_error.unwrap());
                }

                // check if no ref mut arguments passed to a `main()` in a `script` or `predicate`.
                check_no_ref_main(engines, handler, &mains[0]);

                let (entry_fn_id, main_fn_id) = if experimental.new_encoding {
                    if entries.len() != 1 {
                        return Err(handler.emit_err(CompileError::CouldNotGenerateEntry {
                            span: Span::dummy(),
                        }));
                    }
                    (entries[0], mains[0])
                } else {
                    assert!(entries.is_empty());
                    (mains[0], mains[0])
                };

                let main_fn = decl_engine.get(&main_fn_id);
                if !ty_engine.get(main_fn.return_type.type_id()).is_bool() {
                    handler.emit_err(CompileError::PredicateMainDoesNotReturnBool(
                        main_fn.span.clone(),
                    ));
                }

                TyProgramKind::Predicate {
                    entry_function: entry_fn_id,
                    main_function: main_fn_id,
                }
            }
            parsed::TreeType::Script => {
                // A script must have exactly one main function
                if mains.is_empty() {
                    return Err(
                        handler.emit_err(CompileError::NoScriptMainFunction(root.span.clone()))
                    );
                }

                if mains.len() > 1 {
                    let mut last_error = None;
                    for m in mains.iter().skip(1) {
                        let mains_last = decl_engine.get_function(m);
                        last_error = Some(handler.emit_err(
                            CompileError::MultipleDefinitionsOfFunction {
                                name: mains_last.name.clone(),
                                span: mains_last.name.span(),
                            },
                        ));
                    }
                    return Err(last_error.unwrap());
                }

                // check if no ref mut arguments passed to a `main()` in a `script` or `predicate`.
                check_no_ref_main(engines, handler, &mains[0]);

                let (entry_fn_id, main_fn_id) = if experimental.new_encoding {
                    if entries.len() != 1 {
                        return Err(handler.emit_err(CompileError::CouldNotGenerateEntry {
                            span: Span::dummy(),
                        }));
                    }
                    (entries[0], mains[0])
                } else {
                    assert!(entries.is_empty());
                    (mains[0], mains[0])
                };

                // On encoding v0, we cannot accept/return ptrs, slices etc...
                if !experimental.new_encoding {
                    let main_fn = decl_engine.get(&main_fn_id);
                    for p in main_fn.parameters() {
                        if let Some(error) = get_type_not_allowed_error(
                            engines,
                            p.type_argument.type_id(),
                            &p.type_argument,
                            |t| match t {
                                TypeInfo::StringSlice => {
                                    Some(TypeNotAllowedReason::StringSliceInMainParameters)
                                }
                                TypeInfo::RawUntypedSlice => {
                                    Some(TypeNotAllowedReason::NestedSliceReturnNotAllowedInMain)
                                }
                                _ => None,
                            },
                        ) {
                            handler.emit_err(error);
                        }
                    }

                    // Check main return type is valid
                    if let Some(error) = get_type_not_allowed_error(
                        engines,
                        main_fn.return_type.type_id(),
                        &main_fn.return_type,
                        |t| match t {
                            TypeInfo::StringSlice => {
                                Some(TypeNotAllowedReason::StringSliceInMainReturn)
                            }
                            TypeInfo::RawUntypedSlice => {
                                Some(TypeNotAllowedReason::NestedSliceReturnNotAllowedInMain)
                            }
                            _ => None,
                        },
                    ) {
                        // Let main return `raw_slice` directly
                        if !matches!(
                            &*engines.te().get(main_fn.return_type.type_id()),
                            TypeInfo::RawUntypedSlice
                        ) {
                            handler.emit_err(error);
                        }
                    }
                }

                TyProgramKind::Script {
                    entry_function: entry_fn_id,
                    main_function: main_fn_id,
                }
            }
        };

        //configurables and constant cannot be str slice
        for c in configurables.iter() {
            if let Some(error) = get_type_not_allowed_error(
                engines,
                c.return_type,
                &c.type_ascription,
                |t| match t {
                    TypeInfo::StringSlice => Some(TypeNotAllowedReason::StringSliceInConfigurables),
                    TypeInfo::Slice(_) => Some(TypeNotAllowedReason::SliceInConst),
                    _ => None,
                },
            ) {
                handler.emit_err(error);
            }
        }

        // verify all constants
        for decl in root.iter_constants(decl_engine).iter() {
            let decl = decl_engine.get_constant(&decl.decl_id);
            let e =
                get_type_not_allowed_error(engines, decl.return_type, &decl.type_ascription, |t| {
                    match t {
                        TypeInfo::StringSlice => Some(TypeNotAllowedReason::StringSliceInConst),
                        TypeInfo::Slice(_) => Some(TypeNotAllowedReason::SliceInConst),
                        _ => None,
                    }
                });
            if let Some(error) = e {
                handler.emit_err(error);
            }
        }

        Ok((typed_program_kind, declarations, configurables))
    }

    /// All test function declarations within the program.
    pub fn test_fns<'a: 'b, 'b>(
        &'b self,
        decl_engine: &'a DeclEngine,
    ) -> impl 'b + Iterator<Item = (Arc<TyFunctionDecl>, DeclRefFunction)> {
        self.root_module.test_fns_recursive(decl_engine)
    }

    pub fn check_deprecated(&self, engines: &Engines, handler: &Handler) {
        let mut allow_deprecated = AllowDeprecatedState::default();
        self.root_module
            .check_deprecated(engines, handler, &mut allow_deprecated);
    }

    pub fn check_recursive(
        &self,
        engines: &Engines,
        handler: &Handler,
    ) -> Result<(), ErrorEmitted> {
        self.root_module.check_recursive(engines, handler)
    }
}

impl CollectTypesMetadata for TyProgram {
    /// Collect various type information such as unresolved types and types of logged data
    fn collect_types_metadata(
        &self,
        handler: &Handler,
        ctx: &mut CollectTypesMetadataContext,
    ) -> Result<Vec<TypeMetadata>, ErrorEmitted> {
        let decl_engine = ctx.engines.de();
        let mut metadata = vec![];

        // First, look into all entry points that are not unit tests.
        match &self.kind {
            // For scripts and predicates, collect metadata for all the types starting with
            // `main()` as the only entry point
            TyProgramKind::Script {
                entry_function: main_function,
                ..
            }
            | TyProgramKind::Predicate {
                entry_function: main_function,
                ..
            } => {
                let main_function = decl_engine.get_function(main_function);
                metadata.append(&mut main_function.collect_types_metadata(handler, ctx)?);
            }
            // For contracts, collect metadata for all the types starting with each ABI method as
            // an entry point.
            TyProgramKind::Contract {
                abi_entries,
                entry_function: main_function,
            } => {
                if let Some(main_function) = main_function {
                    let entry = decl_engine.get_function(main_function);
                    metadata.append(&mut entry.collect_types_metadata(handler, ctx)?);
                }

                for entry in abi_entries.iter() {
                    let entry = decl_engine.get_function(entry);
                    metadata.append(&mut entry.collect_types_metadata(handler, ctx)?);
                }
            }
            // For libraries, collect metadata for all the types starting with each `pub` node as
            // an entry point. Also dig into all the submodules of a library because nodes in those
            // submodules can also be entry points.
            TyProgramKind::Library { .. } => {
                for module in std::iter::once(&self.root_module).chain(
                    self.root_module
                        .submodules_recursive()
                        .map(|(_, submod)| &*submod.module),
                ) {
                    for node in module.all_nodes.iter() {
                        let is_generic_function = node.is_generic_function(decl_engine);
                        if node.is_public(decl_engine) {
                            let node_metadata = node.collect_types_metadata(handler, ctx)?;
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
        for module in std::iter::once(&self.root_module).chain(
            self.root_module
                .submodules_recursive()
                .map(|(_, submod)| &*submod.module),
        ) {
            for node in module.all_nodes.iter() {
                if node.is_test_function(decl_engine) {
                    metadata.append(&mut node.collect_types_metadata(handler, ctx)?);
                }
            }
        }

        Ok(metadata)
    }
}

#[derive(Clone, Debug)]
pub enum TyProgramKind {
    Contract {
        entry_function: Option<DeclId<TyFunctionDecl>>,
        abi_entries: Vec<DeclId<TyFunctionDecl>>,
    },
    Library {
        name: String,
    },
    Predicate {
        entry_function: DeclId<TyFunctionDecl>,
        main_function: DeclId<TyFunctionDecl>,
    },
    Script {
        entry_function: DeclId<TyFunctionDecl>,
        main_function: DeclId<TyFunctionDecl>,
    },
}

impl TyProgramKind {
    /// The parse tree type associated with this program kind.
    pub fn tree_type(&self) -> parsed::TreeType {
        match self {
            TyProgramKind::Contract { .. } => parsed::TreeType::Contract,
            TyProgramKind::Library { .. } => parsed::TreeType::Library,
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
    declarations: &[TyDecl],
    mains: &[DeclId<TyFunctionDecl>],
) -> Vec<CompileError> {
    let mut errs: Vec<CompileError> = vec![];
    let fn_decls = declarations
        .iter()
        .filter_map(|decl| match decl {
            TyDecl::FunctionDecl(FunctionDecl { decl_id, .. }) => Some(*decl_id),
            _ => None,
        })
        .chain(mains.to_owned());
    let mut err_purity = fn_decls
        .filter_map(|decl_id| {
            let fn_decl = decl_engine.get_function(&decl_id);
            let TyFunctionDecl { purity, name, .. } = &*fn_decl;
            if *purity != Purity::Pure {
                Some(CompileError::ImpureInNonContract { span: name.span() })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    errs.append(&mut err_purity);
    errs
}
