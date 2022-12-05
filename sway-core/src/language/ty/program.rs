use sway_error::error::CompileError;
use sway_types::*;

use crate::{
    declaration_engine::*,
    error::*,
    fuel_prelude::fuel_tx::StorageSlot,
    language::{parsed, ty::*, Purity},
    semantic_analysis::storage_only_types,
    type_system::*,
    Engines,
};

#[derive(Debug, Clone)]
pub struct TyProgram {
    pub kind: TyProgramKind,
    pub root: TyModule,
    pub declarations: Vec<TyDeclaration>,
    pub storage_slots: Vec<StorageSlot>,
    pub logged_types: Vec<(LogId, TypeId)>,
}

impl TyProgram {
    /// Validate the root module given the expected program kind.
    pub fn validate_root(
        engines: Engines<'_>,
        root: &TyModule,
        kind: parsed::TreeType,
        module_span: Span,
    ) -> CompileResult<(TyProgramKind, Vec<TyDeclaration>)> {
        // Extract program-kind-specific properties from the root nodes.
        let mut errors = vec![];
        let mut warnings = vec![];

        let ty_engine = engines.te();

        // Validate all submodules
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
                TyAstNodeContent::Declaration(TyDeclaration::FunctionDeclaration(decl_id)) => {
                    let func = check!(
                        CompileResult::from(de_get_function(decl_id.clone(), &node.span)),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );

                    if func.name.as_str() == "main" {
                        mains.push(func.clone());
                    }

                    if !fn_declarations.insert(func.name.clone()) {
                        errors
                            .push(CompileError::MultipleDefinitionsOfFunction { name: func.name });
                    }

                    declarations.push(TyDeclaration::FunctionDeclaration(decl_id.clone()));
                }
                // ABI entries are all functions declared in impl_traits on the contract type
                // itself.
                TyAstNodeContent::Declaration(TyDeclaration::ImplTrait(decl_id)) => {
                    let TyImplTrait {
                        methods,
                        implementing_for_type_id,
                        span,
                        ..
                    } = check!(
                        CompileResult::from(de_get_impl_trait(decl_id.clone(), &node.span)),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    if matches!(
                        ty_engine.look_up_type_id(implementing_for_type_id),
                        TypeInfo::Contract
                    ) {
                        for method_id in methods {
                            match de_get_function(method_id, &span) {
                                Ok(method) => abi_entries.push(method),
                                Err(err) => errors.push(err),
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
                errors.extend(disallow_impure_functions(&declarations, &mains));
            }

            // `storage` declarations are not allowed in non-contracts
            let storage_decl = declarations
                .iter()
                .find(|decl| matches!(decl, TyDeclaration::StorageDeclaration(_)));

            if let Some(TyDeclaration::StorageDeclaration(decl_id)) = storage_decl {
                let TyStorageDeclaration { span, .. } = check!(
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
            parsed::TreeType::Contract => TyProgramKind::Contract { abi_entries },
            parsed::TreeType::Library { name } => TyProgramKind::Library { name },
            parsed::TreeType::Predicate => {
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
                match ty_engine.look_up_type_id(main_func.return_type) {
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
                    });
                }
                // A script must not return a `raw_ptr` or any type aggregating a `raw_slice`.
                // Directly returning a `raw_slice` is allowed, which will be just mapped to a RETD.
                // TODO: Allow returning nested `raw_slice`s when our spec supports encoding DSTs.
                let main_func = mains.remove(0);
                let main_return_type_info = ty_engine.look_up_type_id(main_func.return_type);
                let nested_types = check!(
                    main_return_type_info
                        .clone()
                        .extract_nested_types(ty_engine, &main_func.return_type_span),
                    vec![],
                    warnings,
                    errors
                );
                if nested_types
                    .iter()
                    .any(|ty| matches!(ty, TypeInfo::RawUntypedPtr))
                {
                    errors.push(CompileError::PointerReturnNotAllowedInMain {
                        span: main_func.return_type_span.clone(),
                    });
                }
                if !matches!(main_return_type_info, TypeInfo::RawUntypedSlice)
                    && nested_types
                        .iter()
                        .any(|ty| matches!(ty, TypeInfo::RawUntypedSlice))
                {
                    errors.push(CompileError::NestedSliceReturnNotAllowedInMain {
                        span: main_func.return_type_span.clone(),
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
                        })
                    }
                }
            }
            _ => (),
        }
        ok((typed_program_kind, declarations), warnings, errors)
    }

    /// Ensures there are no unresolved types or types awaiting resolution in the AST.
    pub(crate) fn collect_types_metadata(
        &mut self,
        ctx: &mut CollectTypesMetadataContext,
    ) -> CompileResult<Vec<TypeMetadata>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        // Get all of the entry points for this tree type. For libraries, that's everything
        // public. For contracts, ABI entries. For scripts and predicates, any function named `main`.
        let metadata = match &self.kind {
            TyProgramKind::Library { .. } => {
                let mut ret = vec![];
                for node in self.root.all_nodes.iter() {
                    let public = check!(
                        node.is_public(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    let is_test = check!(
                        node.is_test_function(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    if public || is_test {
                        ret.append(&mut check!(
                            node.collect_types_metadata(ctx),
                            return err(warnings, errors),
                            warnings,
                            errors
                        ));
                    }
                }
                ret
            }
            TyProgramKind::Script { .. } => {
                let mut data = vec![];
                for node in self.root.all_nodes.iter() {
                    let is_main = check!(
                        node.is_main_function(parsed::TreeType::Script),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    let is_test = check!(
                        node.is_test_function(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    if is_main || is_test {
                        data.append(&mut check!(
                            node.collect_types_metadata(ctx),
                            return err(warnings, errors),
                            warnings,
                            errors
                        ));
                    }
                }
                data
            }
            TyProgramKind::Predicate { .. } => {
                let mut data = vec![];
                for node in self.root.all_nodes.iter() {
                    let is_main = check!(
                        node.is_main_function(parsed::TreeType::Predicate),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    let is_test = check!(
                        node.is_test_function(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    if is_main || is_test {
                        data.append(&mut check!(
                            node.collect_types_metadata(ctx),
                            return err(warnings, errors),
                            warnings,
                            errors
                        ));
                    }
                }
                data
            }
            TyProgramKind::Contract { abi_entries, .. } => {
                let mut data = vec![];
                for node in self.root.all_nodes.iter() {
                    let is_test = check!(
                        node.is_test_function(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    if is_test {
                        data.append(&mut check!(
                            node.collect_types_metadata(ctx),
                            return err(warnings, errors),
                            warnings,
                            errors
                        ));
                    }
                }
                for entry in abi_entries.iter() {
                    data.append(&mut check!(
                        entry.collect_types_metadata(ctx),
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

    pub fn generate_json_abi_program(
        &self,
        type_engine: &TypeEngine,
        types: &mut Vec<fuels_types::TypeDeclaration>,
    ) -> fuels_types::ProgramABI {
        match &self.kind {
            TyProgramKind::Contract { abi_entries, .. } => {
                let functions = abi_entries
                    .iter()
                    .map(|x| x.generate_json_abi_function(type_engine, types))
                    .collect();
                let logged_types = self.generate_json_logged_types(type_engine, types);
                fuels_types::ProgramABI {
                    types: types.to_vec(),
                    functions,
                    logged_types: Some(logged_types),
                }
            }
            TyProgramKind::Script { main_function, .. }
            | TyProgramKind::Predicate { main_function, .. } => {
                let functions = vec![main_function.generate_json_abi_function(type_engine, types)];
                let logged_types = self.generate_json_logged_types(type_engine, types);
                fuels_types::ProgramABI {
                    types: types.to_vec(),
                    functions,
                    logged_types: Some(logged_types),
                }
            }
            _ => fuels_types::ProgramABI {
                types: vec![],
                functions: vec![],
                logged_types: None,
            },
        }
    }

    fn generate_json_logged_types(
        &self,
        type_engine: &TypeEngine,
        types: &mut Vec<fuels_types::TypeDeclaration>,
    ) -> Vec<fuels_types::LoggedType> {
        // A list of all `fuels_types::TypeDeclaration`s needed for the logged types
        let logged_types = self
            .logged_types
            .iter()
            .map(|(_, type_id)| fuels_types::TypeDeclaration {
                type_id: type_id.index(),
                type_field: type_id.get_json_type_str(type_engine, *type_id),
                components: type_id.get_json_type_components(type_engine, types, *type_id),
                type_parameters: type_id.get_json_type_parameters(type_engine, types, *type_id),
            })
            .collect::<Vec<_>>();

        // Add the new types to `types`
        types.extend(logged_types);

        // Generate the JSON data for the logged types
        self.logged_types
            .iter()
            .map(|(log_id, type_id)| fuels_types::LoggedType {
                log_id: **log_id as u64,
                application: fuels_types::TypeApplication {
                    name: "".to_string(),
                    type_id: type_id.index(),
                    type_arguments: type_id.get_json_type_arguments(type_engine, types, *type_id),
                },
            })
            .collect()
    }

    /// All test function declarations within the program.
    pub fn test_fns(&self) -> impl '_ + Iterator<Item = TyFunctionDeclaration> {
        self.root
            .submodules_recursive()
            .flat_map(|(_, submod)| submod.module.test_fns())
            .chain(self.root.test_fns())
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
}

fn disallow_impure_functions(
    declarations: &[TyDeclaration],
    mains: &[TyFunctionDeclaration],
) -> Vec<CompileError> {
    let mut errs: Vec<CompileError> = vec![];
    let fn_decls = declarations
        .iter()
        .filter_map(|decl| match decl {
            TyDeclaration::FunctionDeclaration(decl_id) => {
                match de_get_function(decl_id.clone(), &decl.span()) {
                    Ok(fn_decl) => Some(fn_decl),
                    Err(err) => {
                        errs.push(err);
                        None
                    }
                }
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
