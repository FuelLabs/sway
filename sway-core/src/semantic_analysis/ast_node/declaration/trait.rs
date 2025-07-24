use std::collections::{BTreeMap, HashMap, HashSet};

use ast_elements::type_parameter::GenericTypeParameter;
use parsed_id::ParsedDeclId;
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
    warning::{CompileWarning, Warning},
};
use sway_types::{style::is_upper_camel_case, Ident, Named, Spanned};

use crate::{
    decl_engine::*,
    language::{
        parsed::*,
        ty::{
            self, TyConstantDecl, TyFunctionDecl, TyImplItem, TyTraitDecl, TyTraitFn, TyTraitItem,
            TyTraitType,
        },
        CallPath,
    },
    namespace::{IsExtendingExistingImpl, IsImplInterfaceSurface, IsImplSelf},
    semantic_analysis::{
        declaration::{insert_supertraits_into_namespace, SupertraitOf},
        symbol_collection_context::SymbolCollectionContext,
        AbiMode, TypeCheckAnalysis, TypeCheckAnalysisContext, TypeCheckContext,
        TypeCheckFinalization, TypeCheckFinalizationContext,
    },
    type_system::*,
    Engines,
};

impl TyTraitItem {
    pub(crate) fn collect(
        handler: &Handler,
        engines: &Engines,
        ctx: &mut SymbolCollectionContext,
        item: &TraitItem,
    ) -> Result<(), ErrorEmitted> {
        match item {
            TraitItem::TraitFn(decl_id) => TyTraitFn::collect(handler, engines, ctx, decl_id),
            TraitItem::Constant(decl_id) => TyConstantDecl::collect(handler, engines, ctx, decl_id),
            TraitItem::Type(decl_id) => TyTraitType::collect(handler, engines, ctx, decl_id),
            TraitItem::Error(_, _) => Ok(()),
        }
    }
}

impl TyTraitDecl {
    pub(crate) fn collect(
        handler: &Handler,
        engines: &Engines,
        ctx: &mut SymbolCollectionContext,
        decl_id: &ParsedDeclId<TraitDeclaration>,
    ) -> Result<(), ErrorEmitted> {
        let trait_decl = engines.pe().get_trait(decl_id);
        let decl = Declaration::TraitDeclaration(*decl_id);
        ctx.insert_parsed_symbol(handler, engines, trait_decl.name.clone(), decl.clone())?;

        // A temporary namespace for checking within the trait's scope.
        let _ = ctx.scoped(engines, trait_decl.span.clone(), Some(decl), |scoped_ctx| {
            trait_decl.interface_surface.iter().for_each(|item| {
                let _ = TyTraitItem::collect(handler, engines, scoped_ctx, item);
            });

            trait_decl.methods.iter().for_each(|decl_id| {
                let _ = TyFunctionDecl::collect(handler, engines, scoped_ctx, decl_id);
            });
            Ok(())
        });
        Ok(())
    }

    pub(crate) fn type_check(
        handler: &Handler,
        ctx: TypeCheckContext,
        trait_decl: TraitDeclaration,
    ) -> Result<Self, ErrorEmitted> {
        let TraitDeclaration {
            name,
            type_parameters,
            attributes,
            interface_surface,
            methods,
            supertraits,
            visibility,
            span,
        } = trait_decl;

        if !is_upper_camel_case(name.as_str()) {
            handler.emit_warn(CompileWarning {
                span: name.span(),
                warning_content: Warning::NonClassCaseTraitName { name: name.clone() },
            });
        }

        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();

        // Create a new type parameter for the self type.
        // The span of the `trait_decl` `name` points to the file (use site) in which
        // the trait is getting declared, so we can use it as the `use_site_span`.
        let self_type_param = GenericTypeParameter::new_self_type(engines, name.span());
        let self_type = self_type_param.type_id;

        // A temporary namespace for checking within the trait's scope.
        ctx.with_self_type(Some(self_type))
            .scoped(handler, Some(span.clone()), |ctx| {
                // Type check the type parameters.
                let new_type_parameters = GenericTypeParameter::type_check_type_params(
                    handler,
                    ctx.by_ref(),
                    type_parameters,
                    Some(self_type_param.clone()),
                )?;

                // Recursively make the interface surfaces and methods of the
                // supertraits available to this trait.
                insert_supertraits_into_namespace(
                    handler,
                    ctx.by_ref(),
                    self_type,
                    &supertraits,
                    &SupertraitOf::Trait,
                )?;

                // type check the interface surface
                let mut new_interface_surface = vec![];
                let mut dummy_interface_surface = vec![];

                let mut ids: HashSet<Ident> = HashSet::default();

                for item in interface_surface.clone().into_iter() {
                    let decl_name = match item {
                        TraitItem::TraitFn(_) => None,
                        TraitItem::Constant(_) => None,
                        TraitItem::Type(decl_id) => {
                            let type_decl = engines.pe().get_trait_type(&decl_id).as_ref().clone();
                            let type_decl = ty::TyTraitType::type_check(
                                handler,
                                ctx.by_ref(),
                                type_decl.clone(),
                            )?;
                            let decl_ref = decl_engine.insert(type_decl.clone(), Some(&decl_id));
                            dummy_interface_surface.push(ty::TyImplItem::Type(decl_ref.clone()));
                            new_interface_surface
                                .push(ty::TyTraitInterfaceItem::Type(decl_ref.clone()));

                            Some(type_decl.name)
                        }
                        TraitItem::Error(_, _) => None,
                    };

                    if let Some(decl_name) = decl_name {
                        if !ids.insert(decl_name.clone()) {
                            handler.emit_err(CompileError::MultipleDefinitionsOfName {
                                name: decl_name.clone(),
                                span: decl_name.span(),
                            });
                        }
                    }
                }

                // insert placeholder functions representing the interface surface
                // to allow methods to use those functions
                ctx.insert_trait_implementation(
                    handler,
                    CallPath::ident_to_fullpath(name.clone(), ctx.namespace),
                    new_type_parameters.iter().map(|x| x.into()).collect(),
                    self_type,
                    vec![],
                    &dummy_interface_surface,
                    &span,
                    None,
                    IsImplSelf::No,
                    IsExtendingExistingImpl::No,
                    IsImplInterfaceSurface::No,
                )?;
                let mut dummy_interface_surface = vec![];

                for item in interface_surface.into_iter() {
                    let decl_name = match item {
                        TraitItem::TraitFn(decl_id) => {
                            let method = engines.pe().get_trait_fn(&decl_id);
                            let method = ty::TyTraitFn::type_check(handler, ctx.by_ref(), &method)?;
                            let decl_ref = decl_engine.insert(method.clone(), Some(&decl_id));
                            dummy_interface_surface.push(ty::TyImplItem::Fn(
                                decl_engine
                                    .insert(
                                        method.to_dummy_func(AbiMode::NonAbi, Some(self_type)),
                                        None,
                                    )
                                    .with_parent(decl_engine, (*decl_ref.id()).into()),
                            ));
                            new_interface_surface.push(ty::TyTraitInterfaceItem::TraitFn(decl_ref));
                            Some(method.name.clone())
                        }
                        TraitItem::Constant(decl_id) => {
                            let const_decl = engines.pe().get_constant(&decl_id).as_ref().clone();
                            let const_decl =
                                ty::TyConstantDecl::type_check(handler, ctx.by_ref(), const_decl)?;
                            let decl_ref =
                                ctx.engines.de().insert(const_decl.clone(), Some(&decl_id));
                            new_interface_surface
                                .push(ty::TyTraitInterfaceItem::Constant(decl_ref.clone()));

                            let const_name = const_decl.call_path.suffix.clone();
                            ctx.insert_symbol(
                                handler,
                                const_name.clone(),
                                ty::TyDecl::ConstantDecl(ty::ConstantDecl {
                                    decl_id: *decl_ref.id(),
                                }),
                            )?;

                            Some(const_name)
                        }
                        TraitItem::Type(_) => None,
                        TraitItem::Error(_, _) => {
                            continue;
                        }
                    };

                    if let Some(decl_name) = decl_name {
                        if !ids.insert(decl_name.clone()) {
                            handler.emit_err(CompileError::MultipleDefinitionsOfName {
                                name: decl_name.clone(),
                                span: decl_name.span(),
                            });
                        }
                    }
                }

                // insert placeholder functions representing the interface surface
                // to allow methods to use those functions
                ctx.insert_trait_implementation(
                    handler,
                    CallPath::ident_to_fullpath(name.clone(), ctx.namespace()),
                    new_type_parameters.iter().map(|x| x.into()).collect(),
                    self_type,
                    vec![],
                    &dummy_interface_surface,
                    &span,
                    None,
                    IsImplSelf::No,
                    IsExtendingExistingImpl::Yes,
                    IsImplInterfaceSurface::No,
                )?;

                // Type check the items.
                let mut new_items = vec![];
                for method_decl_id in methods.into_iter() {
                    let method = engines.pe().get_function(&method_decl_id);
                    let method = ty::TyFunctionDecl::type_check(
                        handler,
                        ctx.by_ref(),
                        &method,
                        true,
                        false,
                        Some(self_type_param.type_id),
                    )
                    .unwrap_or_else(|_| ty::TyFunctionDecl::error(&method));
                    new_items.push(ty::TyTraitItem::Fn(
                        decl_engine.insert(method, Some(&method_decl_id)),
                    ));
                }

                let typed_trait_decl = ty::TyTraitDecl {
                    name: name.clone(),
                    type_parameters: new_type_parameters,
                    self_type: TypeParameter::Type(self_type_param),
                    interface_surface: new_interface_surface,
                    items: new_items,
                    supertraits,
                    visibility,
                    attributes,
                    call_path: CallPath::from(name).to_fullpath(ctx.engines(), ctx.namespace()),
                    span,
                };
                Ok(typed_trait_decl)
            })
    }

    /// Retrieves the interface surface and implemented items for this trait.
    pub(crate) fn retrieve_interface_surface_and_implemented_items_for_type(
        &self,
        ctx: TypeCheckContext,
        type_id: TypeId,
        call_path: &CallPath,
    ) -> (InterfaceItemMap, ItemMap) {
        let mut interface_surface_item_refs: InterfaceItemMap = BTreeMap::new();
        let mut impld_item_refs: ItemMap = BTreeMap::new();

        let ty::TyTraitDecl {
            interface_surface, ..
        } = self;

        // Retrieve the interface surface for this trait.
        for item in interface_surface.iter() {
            match item {
                ty::TyTraitInterfaceItem::TraitFn(decl_ref) => {
                    interface_surface_item_refs
                        .insert((decl_ref.name().clone(), type_id), item.clone());
                }
                ty::TyTraitInterfaceItem::Constant(decl_ref) => {
                    interface_surface_item_refs
                        .insert((decl_ref.name().clone(), type_id), item.clone());
                }
                ty::TyTraitInterfaceItem::Type(decl_ref) => {
                    interface_surface_item_refs
                        .insert((decl_ref.name().clone(), type_id), item.clone());
                }
            }
        }

        // Retrieve the implemented items for this type.
        for item in ctx
            .get_items_for_type_and_trait_name(type_id, call_path)
            .into_iter()
        {
            match &item {
                ty::TyTraitItem::Fn(decl_ref) => {
                    impld_item_refs.insert((decl_ref.name().clone(), type_id), item.clone());
                }
                ty::TyTraitItem::Constant(decl_ref) => {
                    impld_item_refs.insert((decl_ref.name().clone(), type_id), item.clone());
                }
                ty::TyTraitItem::Type(decl_ref) => {
                    impld_item_refs.insert((decl_ref.name().clone(), type_id), item.clone());
                }
            };
        }

        (interface_surface_item_refs, impld_item_refs)
    }

    /// Retrieves the interface surface, items, and implemented items for
    /// this trait.
    pub(crate) fn retrieve_interface_surface_and_items_and_implemented_items_for_type(
        &self,
        ctx: &TypeCheckContext,
        type_id: TypeId,
        call_path: &CallPath,
        generic_args: &[GenericArgument],
    ) -> (InterfaceItemMap, ItemMap, ItemMap) {
        let mut interface_surface_item_refs: InterfaceItemMap = BTreeMap::new();
        let mut item_refs: ItemMap = BTreeMap::new();
        let mut impld_item_refs: ItemMap = BTreeMap::new();

        let ty::TyTraitDecl {
            interface_surface,
            items,
            type_parameters,
            ..
        } = self;

        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();

        // Retrieve the interface surface for this trait.
        for item in interface_surface.iter() {
            match item {
                ty::TyTraitInterfaceItem::TraitFn(decl_ref) => {
                    interface_surface_item_refs
                        .insert((decl_ref.name().clone(), type_id), item.clone());
                }
                ty::TyTraitInterfaceItem::Constant(decl_ref) => {
                    interface_surface_item_refs
                        .insert((decl_ref.name().clone(), type_id), item.clone());
                }
                ty::TyTraitInterfaceItem::Type(decl_ref) => {
                    interface_surface_item_refs
                        .insert((decl_ref.name().clone(), type_id), item.clone());
                }
            }
        }

        // Retrieve the trait items for this trait.
        for item in items.iter() {
            match item {
                ty::TyTraitItem::Fn(decl_ref) => {
                    item_refs.insert((decl_ref.name().clone(), type_id), item.clone());
                }
                ty::TyTraitItem::Constant(decl_ref) => {
                    item_refs.insert((decl_ref.name().clone(), type_id), item.clone());
                }
                ty::TyTraitItem::Type(decl_ref) => {
                    item_refs.insert((decl_ref.name().clone(), type_id), item.clone());
                }
            }
        }

        // Retrieve the implemented items for this type.
        let type_mapping = TypeSubstMap::from_type_parameters_and_type_arguments(
            type_parameters
                .iter()
                .map(|t| {
                    let t = t
                        .as_type_parameter()
                        .expect("only works with type parameters");
                    t.type_id
                })
                .collect(),
            generic_args.iter().map(|t| t.type_id()).collect(),
        );

        for item in ctx
            .get_items_for_type_and_trait_name_and_trait_type_arguments(
                type_id,
                call_path,
                generic_args,
            )
            .into_iter()
        {
            match item {
                ty::TyTraitItem::Fn(decl_ref) => {
                    let mut method = (*decl_engine.get_function(&decl_ref)).clone();
                    let name = method.name.clone();
                    let r = if method
                        .subst(&SubstTypesContext::new(
                            engines,
                            &type_mapping,
                            !ctx.code_block_first_pass(),
                        ))
                        .has_changes()
                    {
                        let new_ref = decl_engine
                            .insert(
                                method,
                                decl_engine.get_parsed_decl_id(decl_ref.id()).as_ref(),
                            )
                            .with_parent(decl_engine, (*decl_ref.id()).into());
                        new_ref
                    } else {
                        decl_ref.clone()
                    };
                    impld_item_refs.insert((name, type_id), TyTraitItem::Fn(r));
                }
                ty::TyTraitItem::Constant(decl_ref) => {
                    let mut const_decl = (*decl_engine.get_constant(&decl_ref)).clone();
                    let name = const_decl.call_path.suffix.clone();
                    let r = if const_decl
                        .subst(&SubstTypesContext::new(
                            engines,
                            &type_mapping,
                            !ctx.code_block_first_pass(),
                        ))
                        .has_changes()
                    {
                        decl_engine.insert(
                            const_decl,
                            decl_engine.get_parsed_decl_id(decl_ref.id()).as_ref(),
                        )
                    } else {
                        decl_ref.clone()
                    };
                    impld_item_refs.insert((name, type_id), TyTraitItem::Constant(r));
                }
                ty::TyTraitItem::Type(decl_ref) => {
                    let mut t = (*decl_engine.get_type(&decl_ref)).clone();
                    let name = t.name.clone();
                    let r = if t
                        .subst(&SubstTypesContext::new(
                            engines,
                            &type_mapping,
                            !ctx.code_block_first_pass(),
                        ))
                        .has_changes()
                    {
                        decl_engine
                            .insert(t, decl_engine.get_parsed_decl_id(decl_ref.id()).as_ref())
                    } else {
                        decl_ref.clone()
                    };
                    impld_item_refs.insert((name, type_id), TyTraitItem::Type(r));
                }
            }
        }

        (interface_surface_item_refs, item_refs, impld_item_refs)
    }

    pub(crate) fn insert_interface_surface_and_items_into_namespace(
        &self,
        handler: &Handler,
        mut ctx: TypeCheckContext,
        trait_name: &CallPath,
        type_arguments: &[GenericArgument],
        type_id: TypeId,
    ) {
        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();

        let ty::TyTraitDecl {
            interface_surface,
            items,
            type_parameters,
            ..
        } = self;

        let mut all_items = vec![];

        // Retrieve the trait items for this trait. Transform them into the
        // correct typing for this impl block by using the type parameters from
        // the original trait declaration and the given type arguments.
        let type_mapping = TypeSubstMap::from_type_parameters_and_type_arguments(
            type_parameters
                .iter()
                .map(|t| {
                    let t = t
                        .as_type_parameter()
                        .expect("only works with type parameters");
                    t.type_id
                })
                .collect(),
            type_arguments.iter().map(|t| t.type_id()).collect(),
        );

        let mut const_symbols = HashMap::<Ident, ty::TyDecl>::new();

        for item in interface_surface.iter() {
            match item {
                ty::TyTraitInterfaceItem::TraitFn(decl_ref) => {
                    let mut method = (*decl_engine.get_trait_fn(decl_ref)).clone();
                    method.subst(&SubstTypesContext::new(
                        engines,
                        &type_mapping,
                        !ctx.code_block_first_pass(),
                    ));
                    all_items.push(TyImplItem::Fn(
                        decl_engine
                            .insert(method.to_dummy_func(AbiMode::NonAbi, Some(type_id)), None)
                            .with_parent(ctx.engines.de(), (*decl_ref.id()).into()),
                    ));
                }
                ty::TyTraitInterfaceItem::Constant(decl_ref) => {
                    let const_decl = decl_engine.get_constant(decl_ref);
                    all_items.push(TyImplItem::Constant(decl_ref.clone()));
                    let const_name = const_decl.call_path.suffix.clone();
                    const_symbols.insert(
                        const_name,
                        ty::TyDecl::ConstantDecl(ty::ConstantDecl {
                            decl_id: *decl_ref.id(),
                        }),
                    );
                }
                ty::TyTraitInterfaceItem::Type(decl_ref) => {
                    all_items.push(TyImplItem::Type(decl_ref.clone()));
                }
            }
        }
        for item in items.iter() {
            match item {
                ty::TyTraitItem::Fn(decl_ref) => {
                    let mut method = (*decl_engine.get_function(decl_ref)).clone();
                    method.subst(&SubstTypesContext::new(
                        engines,
                        &type_mapping,
                        !ctx.code_block_first_pass(),
                    ));
                    all_items.push(TyImplItem::Fn(
                        ctx.engines
                            .de()
                            .insert(
                                method,
                                decl_engine.get_parsed_decl_id(decl_ref.id()).as_ref(),
                            )
                            .with_parent(decl_engine, (*decl_ref.id()).into()),
                    ));
                }
                ty::TyTraitItem::Constant(decl_ref) => {
                    let mut const_decl = (*decl_engine.get_constant(decl_ref)).clone();
                    const_decl.subst(&SubstTypesContext::new(
                        engines,
                        &type_mapping,
                        !ctx.code_block_first_pass(),
                    ));
                    let const_name = const_decl.name().clone();
                    let const_has_value = const_decl.value.is_some();
                    let decl_id = decl_engine.insert(
                        const_decl,
                        decl_engine.get_parsed_decl_id(decl_ref.id()).as_ref(),
                    );
                    all_items.push(TyImplItem::Constant(decl_id.clone()));

                    // If this non-interface item has a value, then we want to overwrite the
                    // the previously inserted constant symbol from the interface surface.
                    if const_has_value {
                        const_symbols.insert(
                            const_name,
                            ty::TyDecl::ConstantDecl(ty::ConstantDecl {
                                decl_id: *decl_id.id(),
                            }),
                        );
                    }
                }
                ty::TyTraitItem::Type(decl_ref) => {
                    let mut type_decl = (*decl_engine.get_type(decl_ref)).clone();
                    type_decl.subst(&SubstTypesContext::new(
                        engines,
                        &type_mapping,
                        !ctx.code_block_first_pass(),
                    ));
                    all_items.push(TyImplItem::Type(decl_engine.insert(
                        type_decl,
                        decl_engine.get_parsed_decl_id(decl_ref.id()).as_ref(),
                    )));
                }
            }
        }

        // Insert the constants into the namespace.
        for (name, decl) in const_symbols.into_iter() {
            let _ = ctx.insert_symbol(handler, name, decl);
        }

        // Insert the methods of the trait into the namespace.
        // Specifically do not check for conflicting definitions because
        // this is just a temporary namespace for type checking and
        // these are not actual impl blocks.
        let interface_handler = Handler::default();
        let _ = ctx.insert_trait_implementation(
            &interface_handler,
            trait_name.clone(),
            type_arguments.to_vec(),
            type_id,
            vec![],
            &all_items,
            &trait_name.span(),
            Some(self.span()),
            IsImplSelf::No,
            IsExtendingExistingImpl::No,
            IsImplInterfaceSurface::Yes,
        );
        debug_assert!(!interface_handler.has_errors());
    }
}

impl TypeCheckAnalysis for TyTraitDecl {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        for item in self.items.iter() {
            item.type_check_analyze(handler, ctx)?;
        }
        Ok(())
    }
}

impl TypeCheckFinalization for TyTraitDecl {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        handler.scope(|handler| {
            for item in self.items.iter_mut() {
                let _ = item.type_check_finalize(handler, ctx);
            }
            Ok(())
        })
    }
}
