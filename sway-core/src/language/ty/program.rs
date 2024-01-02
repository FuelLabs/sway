use std::sync::Arc;

use crate::{
    decl_engine::*,
    fuel_prelude::fuel_tx::StorageSlot,
    language::{parsed, ty::*, Purity},
    transform::AllowDeprecatedState,
    type_system::*,
    types::*,
    Engines, ExperimentalFlags,
};

use sway_error::{
    error::{CompileError, TypeNotAllowedReason},
    handler::{ErrorEmitted, Handler},
};
use sway_types::*;

#[derive(Debug, Clone)]
pub struct TyProgram {
    pub kind: TyProgramKind,
    pub root: TyModule,
    pub declarations: Vec<TyDecl>,
    pub configurables: Vec<TyConstantDecl>,
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

impl TyProgram {
    /// Validate the root module given the expected program kind.
    pub fn validate_root(
        handler: &Handler,
        engines: &Engines,
        root: &TyModule,
        kind: parsed::TreeType,
        package_name: &str,
        experimental: ExperimentalFlags,
    ) -> Result<(TyProgramKind, Vec<TyDecl>, Vec<TyConstantDecl>), ErrorEmitted> {
        // Extract program-kind-specific properties from the root nodes.

        let ty_engine = engines.te();
        let decl_engine = engines.de();

        // Validate all submodules
        let mut non_configurables_constants = Vec::<TyConstantDecl>::new();
        let mut configurables = Vec::<TyConstantDecl>::new();
        for (_, submodule) in &root.submodules {
            match Self::validate_root(
                handler,
                engines,
                &submodule.module,
                parsed::TreeType::Library,
                package_name,
                experimental,
            ) {
                Ok(_) => {}
                Err(_) => continue,
            }
        }

        let mut entries = Vec::new();
        let mut declarations = Vec::<TyDecl>::new();
        let mut abi_entries = Vec::new();
        let mut fn_declarations = std::collections::HashSet::new();

        for node in &root.all_nodes {
            match &node.content {
                TyAstNodeContent::Declaration(TyDecl::FunctionDecl(FunctionDecl {
                    name,
                    decl_id,
                    subst_list,
                    decl_span,
                })) => {
                    let func = decl_engine.get_function(decl_id);

                    if matches!(func.kind, TyFunctionDeclKind::Entry) {
                        entries.push(*decl_id);
                    }

                    if !fn_declarations.insert(func.name.clone()) {
                        handler.emit_err(CompileError::MultipleDefinitionsOfFunction {
                            name: func.name.clone(),
                            span: func.name.span(),
                        });
                    }

                    declarations.push(TyDecl::FunctionDecl(FunctionDecl {
                        name: name.clone(),
                        decl_id: *decl_id,
                        subst_list: subst_list.clone(),
                        decl_span: decl_span.clone(),
                    }));
                }
                TyAstNodeContent::Declaration(TyDecl::ConstantDecl(ConstantDecl {
                    decl_id,
                    ..
                })) => {
                    let config_decl = (*decl_engine.get_constant(decl_id)).clone();
                    if config_decl.is_configurable {
                        configurables.push(config_decl);
                    } else {
                        non_configurables_constants.push(config_decl);
                    }
                }
                // ABI entries are all functions declared in impl_traits on the contract type
                // itself, except for ABI supertraits, which do not expose their methods to
                // the user
                TyAstNodeContent::Declaration(TyDecl::ImplTrait(ImplTrait { decl_id, .. })) => {
                    let impl_trait_decl = decl_engine.get_impl_trait(decl_id);
                    let TyImplTrait {
                        items,
                        implementing_for,
                        trait_decl_ref,
                        ..
                    } = &*impl_trait_decl;
                    if matches!(
                        &*ty_engine.get(implementing_for.type_id),
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
                                            let const_decl = decl_engine.get_constant(const_ref);
                                            declarations.push(TyDecl::ConstantDecl(ConstantDecl {
                                                name: const_decl.name().clone(),
                                                decl_id: *const_ref.id(),
                                                decl_span: const_decl.span.clone(),
                                            }));
                                        }
                                        TyImplItem::Type(type_ref) => {
                                            let type_decl = decl_engine.get_type(type_ref);
                                            declarations.push(TyDecl::TraitTypeDecl(
                                                TraitTypeDecl {
                                                    name: type_decl.name().clone(),
                                                    decl_id: *type_ref.id(),
                                                    decl_span: type_decl.span.clone(),
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

            if let Some(TyDecl::StorageDecl(StorageDecl { decl_span, .. })) = storage_decl {
                handler.emit_err(CompileError::StorageDeclarationInNonContract {
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
                    if let TyDecl::StorageDecl(StorageDecl {
                        decl_id,
                        decl_span: _,
                    }) = decl
                    {
                        let storage_decl = decl_engine.get_storage(decl_id);
                        for field in storage_decl.fields.iter() {
                            if let Some(error) = get_type_not_allowed_error(
                                engines,
                                field.type_argument.type_id,
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
                        assert!(entries.len() == 1);
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
                // A predicate must have a main function and that function must return a boolean.
                if entries.is_empty() {
                    return Err(
                        handler.emit_err(CompileError::NoPredicateMainFunction(root.span.clone()))
                    );
                }
                if entries.len() > 1 {
                    let mains_last = decl_engine.get_function(entries.last().unwrap());
                    handler.emit_err(CompileError::MultipleDefinitionsOfFunction {
                        name: mains_last.name.clone(),
                        span: mains_last.name.span(),
                    });
                }
                let main_func_id = entries.remove(0);
                let main_func = decl_engine.get_function(&main_func_id);
                match &*ty_engine.get(main_func.return_type.type_id) {
                    TypeInfo::Boolean => (),
                    _ => {
                        handler.emit_err(CompileError::PredicateMainDoesNotReturnBool(
                            main_func.span.clone(),
                        ));
                    }
                }
                TyProgramKind::Predicate {
                    entry_function: main_func_id,
                }
            }
            parsed::TreeType::Script => {
                // A script must have exactly one main function
                if entries.is_empty() {
                    return Err(
                        handler.emit_err(CompileError::NoScriptMainFunction(root.span.clone()))
                    );
                }

                if entries.len() > 1 {
                    let mains_last = decl_engine.get_function(entries.last().unwrap());
                    handler.emit_err(CompileError::MultipleDefinitionsOfFunction {
                        name: mains_last.name.clone(),
                        span: mains_last.name.span(),
                    });
                }

                // A script must not return a `raw_ptr` or any type aggregating a `raw_slice`.
                // Directly returning a `raw_slice` is allowed, which will be just mapped to a RETD.
                // TODO: Allow returning nested `raw_slice`s when our spec supports encoding DSTs.
                let main_func_decl_id = entries.remove(0);
                let main_func = decl_engine.get_function(&main_func_decl_id);

                for p in main_func.parameters() {
                    if let Some(error) = get_type_not_allowed_error(
                        engines,
                        p.type_argument.type_id,
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
                    main_func.return_type.type_id,
                    &main_func.return_type,
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
                        &*engines.te().get(main_func.return_type.type_id),
                        TypeInfo::RawUntypedSlice
                    ) {
                        handler.emit_err(error);
                    }
                }

                TyProgramKind::Script {
                    entry_function: main_func_decl_id,
                }
            }
        };
        // check if no ref mut arguments passed to a `main()` in a `script` or `predicate`.
        match &typed_program_kind {
            TyProgramKind::Script {
                entry_function: main_function,
                ..
            }
            | TyProgramKind::Predicate {
                entry_function: main_function,
                ..
            } => {
                let main_function = decl_engine.get_function(main_function);
                for param in &main_function.parameters {
                    if param.is_reference && param.is_mutable {
                        handler.emit_err(CompileError::RefMutableNotAllowedInMain {
                            param_name: param.name.clone(),
                            span: param.name.span(),
                        });
                    }
                }
            }
            _ => (),
        }

        //configurables and constant cannot be str slice
        for c in configurables.iter() {
            if let Some(error) = get_type_not_allowed_error(
                engines,
                c.return_type,
                &c.type_ascription,
                |t| match t {
                    TypeInfo::StringSlice => Some(TypeNotAllowedReason::StringSliceInConfigurables),
                    _ => None,
                },
            ) {
                handler.emit_err(error);
            }
        }

        for c in non_configurables_constants.iter() {
            if let Some(error) = get_type_not_allowed_error(
                engines,
                c.return_type,
                &c.type_ascription,
                |t| match t {
                    TypeInfo::StringSlice => Some(TypeNotAllowedReason::StringSliceInConst),
                    _ => None,
                },
            ) {
                handler.emit_err(error);
            }
        }

        Ok((typed_program_kind, declarations, configurables))
    }

    /// All test function declarations within the program.
    pub fn test_fns<'a: 'b, 'b>(
        &'b self,
        decl_engine: &'a DeclEngine,
    ) -> impl '_ + Iterator<Item = (Arc<TyFunctionDecl>, DeclRefFunction)> {
        self.root
            .submodules_recursive()
            .flat_map(|(_, submod)| submod.module.test_fns(decl_engine))
            .chain(self.root.test_fns(decl_engine))
    }

    // /// All entry function declarations within the program.
    // pub fn entry_fns<'a: 'b, 'b>(
    //     &'b self,
    //     decl_engine: &'a DeclEngine,
    //     tree_type: TreeType,
    // ) -> impl '_ + Iterator<Item = DeclRefFunction> {
    //     self.root
    //         .submodules_recursive()
    //         .flat_map(move |(_, submod)| submod.module.entry_fns(decl_engine, tree_type.clone()))
    //         .chain(self.root.entry_fns(decl_engine, tree_type.clone()))
    // }

    pub fn check_deprecated(&self, engines: &Engines, handler: &Handler) {
        let mut allow_deprecated = AllowDeprecatedState::default();
        self.root
            .check_deprecated(engines, handler, &mut allow_deprecated);
    }

    pub fn check_recursive(
        &self,
        engines: &Engines,
        handler: &Handler,
    ) -> Result<(), ErrorEmitted> {
        self.root.check_recursive(engines, handler)
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
                for module in std::iter::once(&self.root).chain(
                    self.root
                        .submodules_recursive()
                        .map(|(_, submod)| &submod.module),
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
        for module in std::iter::once(&self.root).chain(
            self.root
                .submodules_recursive()
                .map(|(_, submod)| &submod.module),
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
    },
    Script {
        entry_function: DeclId<TyFunctionDecl>,
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
