use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::Arc,
};

use sway_error::{
    error::{CompileError, InterfaceName},
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Span, Spanned};

use crate::{
    decl_engine::*,
    engine_threading::*,
    language::{
        parsed::*,
        ty::{self, TyDecl, TyImplItem, TyImplTrait, TyTraitInterfaceItem, TyTraitItem},
        *,
    },
    namespace::{IsExtendingExistingImpl, IsImplSelf, TryInsertingTraitImplOnFailure},
    semantic_analysis::{
        type_check_context::EnforceTypeArguments, AbiMode, ConstShadowingMode,
        TyNodeDepGraphNodeId, TypeCheckAnalysis, TypeCheckAnalysisContext, TypeCheckContext,
        TypeCheckFinalization, TypeCheckFinalizationContext,
    },
    type_system::*,
};

impl TyImplTrait {
    pub(crate) fn type_check_impl_trait(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        impl_trait: ImplTrait,
    ) -> Result<Self, ErrorEmitted> {
        let ImplTrait {
            impl_type_parameters,
            trait_name,
            mut trait_type_arguments,
            mut implementing_for,
            items,
            block_span,
        } = impl_trait;

        let type_engine = ctx.engines.te();
        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();

        // Create a new type parameter for the Self type
        let self_type_param = TypeParameter::new_self_type(engines, implementing_for.span());
        let self_type_id = self_type_param.type_id;

        // create a namespace for the impl
        let mut impl_namespace = ctx.namespace.clone();
        ctx.by_ref()
            .with_const_shadowing_mode(ConstShadowingMode::ItemStyle)
            .with_self_type(Some(self_type_id))
            .allow_functions()
            .scoped(&mut impl_namespace, |mut ctx| {
                // Type check the type parameters
                let new_impl_type_parameters = TypeParameter::type_check_type_params(
                    handler,
                    ctx.by_ref(),
                    impl_type_parameters,
                    Some(self_type_param),
                )?;

                // resolve the types of the trait type arguments
                for type_arg in trait_type_arguments.iter_mut() {
                    type_arg.type_id = ctx.resolve_type(
                        handler,
                        type_arg.type_id,
                        &type_arg.span,
                        EnforceTypeArguments::Yes,
                        None,
                    )?;
                }

                // type check the type that we are implementing for

                implementing_for.type_id = ctx.resolve_type(
                    handler,
                    implementing_for.type_id,
                    &implementing_for.span,
                    EnforceTypeArguments::Yes,
                    None,
                )?;

                // check to see if this type is supported in impl blocks
                type_engine
                    .get(implementing_for.type_id)
                    .expect_is_supported_in_impl_blocks_self(handler, &implementing_for.span)?;

                // check for unconstrained type parameters
                check_for_unconstrained_type_parameters(
                    handler,
                    engines,
                    &new_impl_type_parameters,
                    &trait_type_arguments,
                    implementing_for.type_id,
                )?;

                // Unify the "self" type param and the type that we are implementing for
                handler.scope(|h| {
                    type_engine.unify_with_self(
                        h,
                        engines,
                        implementing_for.type_id,
                        self_type_id,
                        &implementing_for.span,
                        "",
                        None,
                    );
                    Ok(())
                })?;

                // Update the context
                let mut ctx = ctx
                    .with_help_text("")
                    .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None))
                    .with_self_type(Some(implementing_for.type_id));

                let impl_trait = match ctx
                    .namespace
                    .resolve_call_path(handler, engines, &trait_name, ctx.self_type())
                    .ok()
                {
                    Some(ty::TyDecl::TraitDecl(ty::TraitDecl { decl_id, .. })) => {
                        let mut trait_decl = (*decl_engine.get_trait(&decl_id)).clone();

                        // the following essentially is needed to map `Self` to `implementing_for`
                        // during trait decl monomorphization
                        trait_decl
                            .type_parameters
                            .push(trait_decl.self_type.clone());
                        trait_type_arguments.push(implementing_for.clone());

                        // monomorphize the trait declaration
                        ctx.monomorphize(
                            handler,
                            &mut trait_decl,
                            &mut trait_type_arguments,
                            EnforceTypeArguments::Yes,
                            &trait_name.span(),
                        )?;

                        // restore type parameters and type arguments
                        trait_decl.type_parameters.pop();
                        trait_type_arguments.pop();

                        // Insert the interface surface and methods from this trait into
                        // the namespace.
                        trait_decl.insert_interface_surface_and_items_into_namespace(
                            handler,
                            ctx.by_ref(),
                            &trait_name,
                            &trait_type_arguments,
                            implementing_for.type_id,
                        );

                        let new_items = type_check_trait_implementation(
                            handler,
                            ctx.by_ref(),
                            implementing_for.type_id,
                            &new_impl_type_parameters,
                            &trait_decl.type_parameters,
                            &trait_type_arguments,
                            &trait_decl.supertraits,
                            &trait_decl.interface_surface,
                            &trait_decl.items,
                            &items,
                            &trait_name,
                            &trait_decl.span(),
                            &block_span,
                            false,
                        )?;
                        ty::TyImplTrait {
                            impl_type_parameters: new_impl_type_parameters,
                            trait_name: trait_name.clone(),
                            trait_type_arguments,
                            trait_decl_ref: Some(DeclRef::new(
                                trait_decl.name.clone(),
                                decl_id.into(),
                                trait_decl.span.clone(),
                            )),
                            span: block_span,
                            items: new_items,
                            implementing_for,
                        }
                    }
                    Some(ty::TyDecl::AbiDecl(ty::AbiDecl { decl_id, .. })) => {
                        // if you are comparing this with the `impl_trait` branch above, note that
                        // there are no type arguments here because we don't support generic types
                        // in contract ABIs yet (or ever?) due to the complexity of communicating
                        // the ABI layout in the descriptor file.

                        let abi = decl_engine.get_abi(&decl_id);

                        if !type_engine
                            .get(implementing_for.type_id)
                            .eq(&TypeInfo::Contract, engines)
                        {
                            handler.emit_err(CompileError::ImplAbiForNonContract {
                                span: implementing_for.span(),
                                ty: engines.help_out(implementing_for.type_id).to_string(),
                            });
                        }

                        let self_type_param =
                            TypeParameter::new_self_type(engines, abi.span.clone());
                        // Unify the "self" type param from the abi declaration with
                        // the type that we are implementing for.
                        handler.scope(|h| {
                            type_engine.unify_with_self(
                                h,
                                engines,
                                implementing_for.type_id,
                                self_type_param.type_id,
                                &implementing_for.span,
                                "",
                                None,
                            );
                            Ok(())
                        })?;

                        let mut ctx = ctx.with_abi_mode(AbiMode::ImplAbiFn(abi.name.clone(), None));

                        // Insert the interface surface and methods from this trait into
                        // the namespace.
                        let _ = abi.insert_interface_surface_and_items_into_namespace(
                            handler,
                            decl_id,
                            ctx.by_ref(),
                            implementing_for.type_id,
                            None,
                        );

                        let new_items = type_check_trait_implementation(
                            handler,
                            ctx.by_ref(),
                            implementing_for.type_id,
                            &[], // this is empty because abi definitions don't support generics,
                            &[], // this is empty because abi definitions don't support generics,
                            &[], // this is empty because abi definitions don't support generics,
                            &abi.supertraits,
                            &abi.interface_surface,
                            &abi.items,
                            &items,
                            &trait_name,
                            &abi.span(),
                            &block_span,
                            true,
                        )?;
                        ty::TyImplTrait {
                            impl_type_parameters: vec![], // this is empty because abi definitions don't support generics
                            trait_name,
                            trait_type_arguments: vec![], // this is empty because abi definitions don't support generics
                            trait_decl_ref: Some(DeclRef::new(
                                abi.name.clone(),
                                decl_id.into(),
                                abi.span.clone(),
                            )),
                            span: block_span,
                            items: new_items,
                            implementing_for,
                        }
                    }
                    Some(_) | None => {
                        return Err(handler.emit_err(CompileError::UnknownTrait {
                            name: trait_name.suffix.clone(),
                            span: trait_name.span(),
                        }));
                    }
                };
                Ok(impl_trait)
            })
    }

    pub(crate) fn type_check_impl_self(
        handler: &Handler,
        ctx: TypeCheckContext,
        impl_self: ImplSelf,
    ) -> Result<ty::TyDecl, ErrorEmitted> {
        let ImplSelf {
            impl_type_parameters,
            mut implementing_for,
            items,
            block_span,
        } = impl_self;

        let type_engine = ctx.engines.te();
        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();

        // create the namespace for the impl
        let mut impl_namespace = ctx.namespace.clone();
        ctx.with_const_shadowing_mode(ConstShadowingMode::ItemStyle)
            .allow_functions()
            .scoped(&mut impl_namespace, |mut ctx| {
                // Create a new type parameter for the "self type".
                let self_type_param =
                    TypeParameter::new_self_type(engines, implementing_for.span());
                let self_type_id = self_type_param.type_id;

                // create the trait name
                let trait_name = CallPath {
                    prefixes: vec![],
                    suffix: match &&*type_engine.get(implementing_for.type_id) {
                        TypeInfo::Custom {
                            qualified_call_path: call_path,
                            ..
                        } => call_path.call_path.suffix.clone(),
                        _ => Ident::new_with_override("r#Self".into(), implementing_for.span()),
                    },
                    is_absolute: false,
                };

                // Type check the type parameters.
                let new_impl_type_parameters = TypeParameter::type_check_type_params(
                    handler,
                    ctx.by_ref(),
                    impl_type_parameters,
                    Some(self_type_param),
                )?;

                // type check the type that we are implementing for
                implementing_for.type_id = ctx.resolve_type(
                    handler,
                    implementing_for.type_id,
                    &implementing_for.span,
                    EnforceTypeArguments::Yes,
                    None,
                )?;

                // check to see if this type is supported in impl blocks
                type_engine
                    .get(implementing_for.type_id)
                    .expect_is_supported_in_impl_blocks_self(handler, &implementing_for.span)?;

                // check for unconstrained type parameters
                check_for_unconstrained_type_parameters(
                    handler,
                    engines,
                    &new_impl_type_parameters,
                    &[],
                    implementing_for.type_id,
                )?;

                implementing_for.type_id.check_type_parameter_bounds(
                    handler,
                    ctx.by_ref(),
                    &implementing_for.span,
                    None,
                )?;

                // Unify the "self" type param and the type that we are implementing for
                handler.scope(|h| {
                    type_engine.unify_with_self(
                        h,
                        engines,
                        implementing_for.type_id,
                        self_type_id,
                        &implementing_for.span,
                        "",
                        None,
                    );
                    Ok(())
                })?;

                let mut ctx = ctx
                    .with_help_text("")
                    .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None));

                // type check the items inside of the impl block
                let mut new_items = vec![];

                handler.scope(|handler| {
                    for item in items.iter() {
                        match item {
                            ImplItem::Fn(fn_decl_id) => {
                                let fn_decl = engines.pe().get_function(fn_decl_id);
                                let fn_decl = match ty::TyFunctionDecl::type_check_signature(
                                    handler,
                                    ctx.by_ref(),
                                    &fn_decl,
                                    true,
                                    true,
                                    Some(implementing_for.type_id),
                                ) {
                                    Ok(res) => res,
                                    Err(_) => continue,
                                };
                                new_items.push(TyImplItem::Fn(decl_engine.insert(fn_decl)));
                            }
                            ImplItem::Constant(decl_id) => {
                                let const_decl =
                                    engines.pe().get_constant(decl_id).as_ref().clone();
                                let const_decl = match ty::TyConstantDecl::type_check(
                                    handler,
                                    ctx.by_ref(),
                                    const_decl,
                                ) {
                                    Ok(res) => res,
                                    Err(_) => continue,
                                };
                                let decl_ref = decl_engine.insert(const_decl);
                                new_items.push(TyImplItem::Constant(decl_ref.clone()));

                                ctx.insert_symbol(
                                    handler,
                                    decl_ref.name().clone(),
                                    ty::TyDecl::ConstantDecl(ty::ConstantDecl {
                                        name: decl_ref.name().clone(),
                                        decl_id: *decl_ref.id(),
                                        decl_span: decl_ref.span().clone(),
                                    }),
                                )?;
                            }
                            ImplItem::Type(decl_id) => {
                                let type_decl =
                                    engines.pe().get_trait_type(decl_id).as_ref().clone();
                                let type_decl = match ty::TyTraitType::type_check(
                                    handler,
                                    ctx.by_ref(),
                                    type_decl.clone(),
                                ) {
                                    Ok(res) => res,
                                    Err(_) => continue,
                                };
                                let decl_ref = decl_engine.insert(type_decl);
                                new_items.push(TyImplItem::Type(decl_ref.clone()));
                            }
                        }
                    }

                    let mut impl_trait = ty::TyImplTrait {
                        impl_type_parameters: new_impl_type_parameters,
                        trait_name,
                        trait_type_arguments: vec![], // this is empty because impl selfs don't support generics on the "Self" trait,
                        trait_decl_ref: None,
                        span: block_span,
                        items: new_items,
                        implementing_for,
                    };

                    ctx.insert_trait_implementation(
                        handler,
                        impl_trait.trait_name.clone(),
                        impl_trait.trait_type_arguments.clone(),
                        impl_trait.implementing_for.type_id,
                        &impl_trait.items,
                        &impl_trait.span,
                        impl_trait
                            .trait_decl_ref
                            .as_ref()
                            .map(|decl_ref| decl_ref.decl_span().clone()),
                        IsImplSelf::Yes,
                        IsExtendingExistingImpl::No,
                    )?;

                    // Now lets do a partial type check of the body of the functions (while deferring full
                    // monomorphization of function applications). We will use this tree to perform type check
                    // analysis (mainly dependency analysis), and re-type check the items ordered by dependency.
                    let mut defer_ctx = ctx.by_ref().with_defer_monomorphization();

                    let new_items = &impl_trait.items;
                    for (item, new_item) in items.clone().into_iter().zip(new_items) {
                        match (item, new_item) {
                            (ImplItem::Fn(fn_decl_id), TyTraitItem::Fn(decl_ref)) => {
                                let fn_decl = engines.pe().get_function(&fn_decl_id);
                                let mut ty_fn_decl =
                                    (*decl_engine.get_function(decl_ref.id())).clone();
                                let new_ty_fn_decl = match ty::TyFunctionDecl::type_check_body(
                                    handler,
                                    defer_ctx.by_ref(),
                                    &fn_decl,
                                    &mut ty_fn_decl,
                                ) {
                                    Ok(res) => res,
                                    Err(_) => continue,
                                };
                                decl_engine.replace(*decl_ref.id(), new_ty_fn_decl);
                            }
                            (ImplItem::Constant(_const_decl), TyTraitItem::Constant(_decl_ref)) => {
                                // Already processed.
                            }
                            (ImplItem::Type(_type_decl), TyTraitItem::Type(_decl_ref)) => {
                                // Already processed.
                            }
                            _ => unreachable!(),
                        }
                    }

                    let impl_trait_decl = decl_engine.insert(impl_trait.clone()).into();

                    // First lets perform an analysis pass.
                    // This returns a vector with ordered indexes to the items in the order that they
                    // should be processed.
                    let ordered_node_indices_opt =
                        if let TyDecl::ImplTrait(impl_trait) = &impl_trait_decl {
                            ty::TyImplTrait::type_check_analyze_impl_self_items(
                                handler,
                                ctx.by_ref(),
                                impl_trait,
                            )?
                        } else {
                            unreachable!();
                        };

                    // In case there was any issue processing the dependency graph, then lets just
                    // process them in the original order.
                    let ordered_node_indices: Vec<_> = match ordered_node_indices_opt {
                        Some(value) => value.iter().map(|n| n.index()).collect(),
                        None => (0..new_items.len()).collect(),
                    };

                    // Now lets type check the body of the functions (for real this time).
                    for idx in ordered_node_indices {
                        match (&items[idx], &new_items[idx]) {
                            (ImplItem::Fn(fn_decl_id), TyTraitItem::Fn(decl_ref)) => {
                                let fn_decl = engines.pe().get_function(fn_decl_id);
                                let mut ty_fn_decl =
                                    (*decl_engine.get_function(decl_ref.id())).clone();
                                let new_ty_fn_decl = match ty::TyFunctionDecl::type_check_body(
                                    handler,
                                    ctx.by_ref(),
                                    &fn_decl,
                                    &mut ty_fn_decl,
                                ) {
                                    Ok(res) => res,
                                    Err(_) => continue,
                                };
                                decl_engine.replace(*decl_ref.id(), new_ty_fn_decl);
                            }
                            (ImplItem::Constant(_const_decl), TyTraitItem::Constant(_decl_ref)) => {
                                // Already processed.
                            }
                            (ImplItem::Type(_type_decl), TyTraitItem::Type(_decl_ref)) => {
                                // Already processed.
                            }
                            _ => unreachable!(),
                        }
                    }

                    let impl_trait_decl: ty::TyDecl = decl_engine.insert(impl_trait.clone()).into();

                    let mut finalizing_ctx =
                        TypeCheckFinalizationContext::new(ctx.engines, ctx.by_ref());
                    for item in new_items {
                        match item {
                            TyTraitItem::Fn(decl_ref) => {
                                let mut fn_decl =
                                    (*decl_engine.get_function(decl_ref.id())).clone();
                                let _ = fn_decl.type_check_finalize(handler, &mut finalizing_ctx);
                                decl_engine.replace(*decl_ref.id(), fn_decl);
                            }
                            TyTraitItem::Constant(decl_ref) => {
                                let mut const_decl =
                                    (*decl_engine.get_constant(decl_ref.id())).clone();
                                let _ =
                                    const_decl.type_check_finalize(handler, &mut finalizing_ctx);
                                decl_engine.replace(*decl_ref.id(), const_decl);
                            }
                            _ => {}
                        }
                    }

                    impl_trait.items.iter_mut().for_each(|item| {
                        item.replace_implementing_type(engines, impl_trait_decl.clone())
                    });

                    if let TyDecl::ImplTrait(impl_trait_id) = &impl_trait_decl {
                        decl_engine.replace(impl_trait_id.decl_id, impl_trait);
                    }

                    Ok(impl_trait_decl)
                })
            })
    }

    pub(crate) fn type_check_analyze_impl_self_items(
        handler: &Handler,
        ctx: TypeCheckContext,
        impl_self: &ty::ImplTrait,
    ) -> Result<Option<Vec<TyNodeDepGraphNodeId>>, ErrorEmitted> {
        let engines = ctx.engines;
        handler.scope(|handler| {
            let mut analysis_ctx = TypeCheckAnalysisContext::new(engines);
            let _ = impl_self.type_check_analyze(handler, &mut analysis_ctx);

            // Build a sub graph that just contains the items for this impl trait.
            let impl_trait_node_index = analysis_ctx.nodes.get(&impl_self.decl_id.unique_id());
            let sub_graph = analysis_ctx.get_sub_graph(
                *impl_trait_node_index.expect("expected a valid impl trait node id"),
            );

            // Do a topological sort to compute an ordered list of nodes (by function calls).
            let sorted = match petgraph::algo::toposort(&sub_graph, None) {
                Ok(value) => Some(value),
                // If the dependency graph contains cycles, then this means there are recursive
                // function calls, which we will report later.
                Err(_) => None,
            };

            Ok(sorted)
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn type_check_trait_implementation(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    implementing_for: TypeId,
    impl_type_parameters: &[TypeParameter],
    trait_type_parameters: &[TypeParameter],
    trait_type_arguments: &[TypeArgument],
    trait_supertraits: &[Supertrait],
    trait_interface_surface: &[TyTraitInterfaceItem],
    trait_items: &[TyImplItem],
    impl_items: &[ImplItem],
    trait_name: &CallPath,
    trait_decl_span: &Span,
    block_span: &Span,
    is_contract: bool,
) -> Result<Vec<TyImplItem>, ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let decl_engine = ctx.engines.de();
    let engines = ctx.engines();

    // Check to see if the type that we are implementing for implements the
    // supertraits of this trait.
    ctx.namespace
        .module_mut()
        .current_items_mut()
        .implemented_traits
        .check_if_trait_constraints_are_satisfied_for_type(
            handler,
            implementing_for,
            &trait_supertraits
                .iter()
                .map(|x| x.into())
                .collect::<Vec<_>>(),
            block_span,
            engines,
            TryInsertingTraitImplOnFailure::Yes,
        )?;

    for (type_arg, type_param) in trait_type_arguments.iter().zip(trait_type_parameters) {
        type_arg.type_id.check_type_parameter_bounds(
            handler,
            ctx.by_ref(),
            &type_arg.span(),
            Some(type_param.clone()),
        )?;
    }

    // This map keeps track of the remaining functions in the interface surface
    // that still need to be implemented for the trait to be fully implemented.
    let mut method_checklist: BTreeMap<Ident, Arc<ty::TyTraitFn>> = BTreeMap::new();

    // This map keeps track of the remaining constants in the interface surface
    // that still need to be implemented for the trait to be fully implemented.
    let mut constant_checklist: BTreeMap<Ident, Arc<ty::TyConstantDecl>> = BTreeMap::new();

    // This map keeps track of the remaining types in the interface surface
    // that still need to be implemented for the trait to be fully implemented.
    let mut type_checklist: BTreeMap<Ident, Arc<ty::TyTraitType>> = BTreeMap::new();

    // This map keeps track of the interface declaration id's of the trait
    // definition.
    let mut interface_item_refs: InterfaceItemMap = BTreeMap::new();

    // This map keeps track of the new declaration ids of the implemented
    // interface surface.
    let mut impld_item_refs: ItemMap = BTreeMap::new();

    // This map keeps track of the stub declaration id's of the supertraits.
    let mut supertrait_interface_item_refs: InterfaceItemMap = BTreeMap::new();

    // This map keeps track of the new declaration ids of the supertraits.
    let mut supertrait_impld_item_refs: ItemMap = BTreeMap::new();

    // Insert the implemented methods for the supertraits into this namespace
    // so that the methods defined in the impl block can use them.
    //
    // We purposefully do not check for errors here because this is a temporary
    // namespace and not a real impl block defined by the user.
    if !trait_supertraits.is_empty() {
        // Gather the supertrait "stub_method_refs" and "impld_method_refs".
        let (this_supertrait_stub_method_refs, this_supertrait_impld_method_refs) =
            handle_supertraits(handler, ctx.by_ref(), implementing_for, trait_supertraits)?;

        let _ = ctx.insert_trait_implementation(
            &Handler::default(),
            trait_name.clone(),
            trait_type_arguments.to_vec(),
            implementing_for,
            &this_supertrait_impld_method_refs
                .values()
                .cloned()
                .collect::<Vec<_>>(),
            &trait_name.span(),
            Some(trait_decl_span.clone()),
            IsImplSelf::No,
            IsExtendingExistingImpl::No,
        );

        supertrait_interface_item_refs = this_supertrait_stub_method_refs;
        supertrait_impld_item_refs = this_supertrait_impld_method_refs;
    }

    for item in trait_interface_surface.iter() {
        match item {
            TyTraitInterfaceItem::TraitFn(decl_ref) => {
                let method = decl_engine.get_trait_fn(decl_ref);
                let name = method.name.clone();
                method_checklist.insert(name.clone(), method);

                // After monomorphization (and typechecking trait implementation
                // works on a monomorphized version of a trait declaration), the
                // newly produced [decl_id]s for interface methods can only be
                // traced back to the original (non-monomorphized) trait
                // declaration, which means that during [decl_id] substitutions
                // of trait interface dummy functions for the corresponding
                // interface method _implementations_, we need to refer to the
                // original [decl_id]s of the non-monomorphized trait
                // declaration.
                let interface_item_parents = decl_engine.find_all_parents(engines, decl_ref.id());
                match interface_item_parents.len() {
                    0 => { interface_item_refs.insert((name, implementing_for), item.clone()); },
                    1 => match interface_item_parents[0] {
                            AssociatedItemDeclId::TraitFn(parent_decl_id) => {
                                let parent_interface_item =
                                    TyTraitInterfaceItem::TraitFn(DeclRef::new(name.clone(), parent_decl_id, decl_ref.span()));
                                interface_item_refs.insert((name, implementing_for), parent_interface_item);
                            }
                            _ => return Err(handler.emit_err(CompileError::Internal(
                                    "A trait interface method's parent is expected to be another trait interface method",
                                    name.span())))
                         }
                    _ => return Err(handler.emit_err(CompileError::Internal(
                            "A trait interface method is expected to have zero parents or one parent in the Declaration Engine",
                            name.span())))
                }
            }
            TyTraitInterfaceItem::Constant(decl_ref) => {
                let constant = decl_engine.get_constant(decl_ref);
                let name = constant.call_path.suffix.clone();
                constant_checklist.insert(name.clone(), constant);
                interface_item_refs.insert((name, implementing_for), item.clone());
            }
            TyTraitInterfaceItem::Type(decl_ref) => {
                let ty = decl_engine.get_type(decl_ref);
                let name = ty.name.clone();
                type_checklist.insert(name.clone(), ty);
                interface_item_refs.insert((name, implementing_for), item.clone());
            }
        }
    }

    let mut trait_type_mapping =
        TypeSubstMap::from_type_parameters_and_type_arguments(vec![], vec![]);

    for item in impl_items {
        match item {
            ImplItem::Fn(_) => {}
            ImplItem::Constant(_) => {}
            ImplItem::Type(decl_id) => {
                let type_decl = engines.pe().get_trait_type(decl_id);
                let mut type_decl = type_check_type_decl(
                    handler,
                    ctx.by_ref(),
                    &type_decl,
                    trait_name,
                    implementing_for,
                    is_contract,
                    &impld_item_refs,
                    &type_checklist,
                )
                .unwrap_or_else(|_| {
                    ty::TyTraitType::error(ctx.engines(), type_decl.as_ref().clone())
                });

                type_decl.subst(&trait_type_mapping, engines);

                // Remove this type from the checklist.
                let name = type_decl.name.clone();
                type_checklist.remove(&name);

                // Add this type to the "impld decls".
                let decl_ref = decl_engine.insert(type_decl.clone());
                impld_item_refs.insert((name, implementing_for), TyTraitItem::Type(decl_ref));

                let old_type_decl_info1 = TypeInfo::TraitType {
                    name: type_decl.name.clone(),
                    trait_type_id: implementing_for,
                };
                let old_type_decl_info2 = TypeInfo::TraitType {
                    name: type_decl.name.clone(),
                    trait_type_id: type_engine.insert(
                        engines,
                        TypeInfo::UnknownGeneric {
                            // Using Span::dummy just to match the type substitution, type is not used anywhere else.
                            name: Ident::new_with_override("Self".into(), Span::dummy()),
                            trait_constraints: VecSet(vec![]),
                        },
                        None,
                    ),
                };
                trait_type_mapping.extend(TypeSubstMap::from_type_parameters_and_type_arguments(
                    vec![type_engine.insert(
                        engines,
                        old_type_decl_info1,
                        type_decl.name.span().source_id(),
                    )],
                    vec![type_decl.ty.clone().unwrap().type_id],
                ));
                trait_type_mapping.extend(TypeSubstMap::from_type_parameters_and_type_arguments(
                    vec![type_engine.insert(
                        engines,
                        old_type_decl_info2,
                        type_decl.name.span().source_id(),
                    )],
                    vec![type_decl.ty.clone().unwrap().type_id],
                ));
            }
        }
    }

    for item in impl_items {
        match item {
            ImplItem::Fn(impl_method_id) => {
                let impl_method = engines.pe().get_function(impl_method_id);
                let mut impl_method = type_check_impl_method(
                    handler,
                    ctx.by_ref().with_type_subst(&trait_type_mapping),
                    implementing_for,
                    impl_type_parameters,
                    &impl_method,
                    trait_name,
                    is_contract,
                    &impld_item_refs,
                    &method_checklist,
                )
                .unwrap_or_else(|_| ty::TyFunctionDecl::error(&impl_method));

                impl_method.subst(&trait_type_mapping, engines);

                // Remove this method from the checklist.
                let name = impl_method.name.clone();
                method_checklist.remove(&name);

                // Add this method to the "impld items".
                let decl_ref = decl_engine.insert(impl_method);
                impld_item_refs.insert((name, implementing_for), TyTraitItem::Fn(decl_ref));
            }
            ImplItem::Constant(decl_id) => {
                let const_decl = engines.pe().get_constant(decl_id).as_ref().clone();
                let mut const_decl = type_check_const_decl(
                    handler,
                    ctx.by_ref().with_type_subst(&trait_type_mapping),
                    &const_decl,
                    trait_name,
                    is_contract,
                    &impld_item_refs,
                    &constant_checklist,
                )
                .unwrap_or_else(|_| ty::TyConstantDecl::error(ctx.engines(), const_decl.clone()));

                const_decl.subst(&trait_type_mapping, engines);

                // Remove this constant from the checklist.
                let name = const_decl.call_path.suffix.clone();
                constant_checklist.remove(&name);

                // Add this constant to the "impld decls".
                let decl_ref = decl_engine.insert(const_decl);
                impld_item_refs.insert((name, implementing_for), TyTraitItem::Constant(decl_ref));
            }
            ImplItem::Type(_) => {}
        }
    }

    let mut all_items_refs: Vec<TyImplItem> = impld_item_refs.values().cloned().collect();

    // Retrieve the methods defined on the trait declaration and transform
    // them into the correct typing for this impl block by using the type
    // parameters from the original trait declaration and the type arguments of
    // the trait name in the current impl block that we are type checking and
    // using the stub decl ids from the interface surface and the new
    // decl ids from the newly implemented methods.
    let mut type_mapping = TypeSubstMap::from_type_parameters_and_type_arguments(
        trait_type_parameters
            .iter()
            .map(|type_param| type_param.type_id)
            .collect(),
        trait_type_arguments
            .iter()
            .map(|type_arg| type_arg.type_id)
            .collect(),
    );
    type_mapping.extend(trait_type_mapping);

    interface_item_refs.extend(supertrait_interface_item_refs);
    impld_item_refs.extend(supertrait_impld_item_refs);
    let decl_mapping = DeclMapping::from_interface_and_item_and_impld_decl_refs(
        interface_item_refs,
        BTreeMap::new(),
        impld_item_refs,
    );
    for item in trait_items.iter() {
        match item {
            TyImplItem::Fn(decl_ref) => {
                let mut method = (*decl_engine.get_function(decl_ref)).clone();

                // We need to add impl type parameters to the method's type parameters
                // so that in-line monomorphization can complete.
                //
                // We also need to add impl type parameters to the method's type
                // parameters so the type constraints are correctly applied to the method.
                //
                // NOTE: this is a semi-hack that is used to force monomorphization of
                // trait methods that contain a generic defined in the parent impl...
                // without stuffing the generic into the method's type parameters, its
                // not currently possible to monomorphize on that generic at function
                // application time.
                method.type_parameters.append(
                    &mut impl_type_parameters
                        .iter()
                        .cloned()
                        .map(|mut t| {
                            t.is_from_parent = true;
                            t
                        })
                        .collect::<Vec<_>>(),
                );

                method.implementing_for_typeid = Some(implementing_for);
                method.replace_decls(&decl_mapping, handler, &mut ctx)?;
                method.subst(&type_mapping, engines);
                all_items_refs.push(TyImplItem::Fn(
                    decl_engine
                        .insert(method)
                        .with_parent(decl_engine, (*decl_ref.id()).into()),
                ));
            }
            TyImplItem::Constant(decl_ref) => {
                let mut const_decl = (*decl_engine.get_constant(decl_ref)).clone();
                const_decl.replace_decls(&decl_mapping, handler, &mut ctx)?;
                const_decl.subst(&type_mapping, engines);
                all_items_refs.push(TyImplItem::Constant(decl_engine.insert(const_decl)));
            }
            TyImplItem::Type(decl_ref) => {
                let mut type_decl = (*decl_engine.get_type(decl_ref)).clone();
                type_decl.subst(&type_mapping, engines);
                all_items_refs.push(TyImplItem::Type(decl_engine.insert(type_decl.clone())));
            }
        }
    }

    handler.scope(|handler| {
        // check that the implementation checklist is complete
        if !method_checklist.is_empty() {
            handler.emit_err(CompileError::MissingInterfaceSurfaceMethods {
                span: block_span.clone(),
                missing_functions: method_checklist.into_keys().collect::<Vec<_>>(),
            });
        }

        if !constant_checklist.is_empty() {
            handler.emit_err(CompileError::MissingInterfaceSurfaceConstants {
                span: block_span.clone(),
                missing_constants: constant_checklist.into_keys().collect::<Vec<_>>(),
            });
        }

        if !type_checklist.is_empty() {
            handler.emit_err(CompileError::MissingInterfaceSurfaceTypes {
                span: block_span.clone(),
                missing_types: type_checklist.into_keys().collect::<Vec<_>>(),
            });
        }

        Ok(all_items_refs)
    })
}

#[allow(clippy::too_many_arguments)]
fn type_check_impl_method(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    implementing_for: TypeId,
    impl_type_parameters: &[TypeParameter],
    impl_method: &FunctionDeclaration,
    trait_name: &CallPath,
    is_contract: bool,
    impld_item_refs: &ItemMap,
    method_checklist: &BTreeMap<Ident, Arc<ty::TyTraitFn>>,
) -> Result<ty::TyFunctionDecl, ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    let mut ctx = ctx
        .by_ref()
        .with_help_text("")
        .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None));

    let interface_name = || -> InterfaceName {
        if is_contract {
            InterfaceName::Abi(trait_name.suffix.clone())
        } else {
            InterfaceName::Trait(trait_name.suffix.clone())
        }
    };

    // type check the function declaration
    let mut impl_method = ty::TyFunctionDecl::type_check(
        handler,
        ctx.by_ref(),
        impl_method,
        true,
        false,
        Some(implementing_for),
    )?;

    // Ensure that there aren't multiple definitions of this function impl'd
    if impld_item_refs.contains_key(&(impl_method.name.clone(), implementing_for)) {
        return Err(
            handler.emit_err(CompileError::MultipleDefinitionsOfFunction {
                name: impl_method.name.clone(),
                span: impl_method.name.span(),
            }),
        );
    }

    // Ensure that the method checklist contains this function.
    let mut impl_method_signature = match method_checklist.get(&impl_method.name) {
        Some(trait_fn) => (**trait_fn).clone(),
        None => {
            return Err(
                handler.emit_err(CompileError::FunctionNotAPartOfInterfaceSurface {
                    name: impl_method.name.clone(),
                    interface_name: interface_name(),
                    span: impl_method.name.span(),
                }),
            );
        }
    };

    // ensure this fn decl's parameters and signature lines up with the one
    // in the trait
    if impl_method.parameters.len() != impl_method_signature.parameters.len() {
        return Err(handler.emit_err(
            CompileError::IncorrectNumberOfInterfaceSurfaceFunctionParameters {
                span: impl_method.parameters_span(),
                fn_name: impl_method.name.clone(),
                interface_name: interface_name(),
                num_parameters: impl_method_signature.parameters.len(),
                provided_parameters: impl_method.parameters.len(),
            },
        ));
    }

    handler.scope(|handler| {
        // Ensure that the method decl's parameters match the stub method's parameters
        for (impl_method_signature_param, impl_method_param) in impl_method_signature
            .parameters
            .iter_mut()
            .zip(&mut impl_method.parameters)
        {
            // TODO use trait constraints as part of the type here to
            // implement trait constraint solver */
            // Check if we have a non-ref mutable argument. That's not allowed.
            if impl_method_signature_param.is_mutable && !impl_method_signature_param.is_reference {
                handler.emit_err(CompileError::MutableParameterNotSupported {
                    param_name: impl_method_signature.name.clone(),
                    span: impl_method_signature.name.span(),
                });
            }

            // check if reference / mutability of the parameters is incompatible
            if impl_method_param.is_mutable != impl_method_signature_param.is_mutable
                || impl_method_param.is_reference != impl_method_signature_param.is_reference
            {
                handler.emit_err(CompileError::ParameterRefMutabilityMismatch {
                    span: impl_method_param.mutability_span.clone(),
                });
            }

            // this subst is required to replace associated types, namely TypeInfo::TraitType.
            let mut impl_method_param_type_id = impl_method_param.type_argument.type_id;
            impl_method_param_type_id.subst(&ctx.type_subst(), engines);

            let mut impl_method_signature_param_type_id =
                impl_method_signature_param.type_argument.type_id;
            impl_method_signature_param_type_id.subst(&ctx.type_subst(), engines);

            if !type_engine.get(impl_method_param_type_id).eq(
                &type_engine.get(impl_method_signature_param_type_id),
                engines,
            ) {
                handler.emit_err(CompileError::MismatchedTypeInInterfaceSurface {
                    interface_name: interface_name(),
                    span: impl_method_param.type_argument.span.clone(),
                    decl_type: "function".to_string(),
                    given: engines.help_out(impl_method_param_type_id).to_string(),
                    expected: engines
                        .help_out(impl_method_signature_param_type_id)
                        .to_string(),
                });
                continue;
            }
        }

        // check to see if the purity of the function declaration is the same
        // as the purity of the function signature
        if impl_method.purity != impl_method_signature.purity {
            handler.emit_err(if impl_method_signature.purity == Purity::Pure {
                CompileError::TraitDeclPureImplImpure {
                    fn_name: impl_method.name.clone(),
                    interface_name: interface_name(),
                    attrs: impl_method.purity.to_attribute_syntax(),
                    span: impl_method.span.clone(),
                }
            } else {
                CompileError::TraitImplPurityMismatch {
                    fn_name: impl_method.name.clone(),
                    interface_name: interface_name(),
                    attrs: impl_method_signature.purity.to_attribute_syntax(),
                    span: impl_method.span.clone(),
                }
            });
        }

        // check there is no mismatch of payability attributes
        // between the method signature and the method implementation
        use crate::transform::AttributeKind::Payable;
        let impl_method_signature_payable = impl_method_signature.attributes.contains_key(&Payable);
        let impl_method_payable = impl_method.attributes.contains_key(&Payable);
        match (impl_method_signature_payable, impl_method_payable) {
            (true, false) =>
            // implementation does not have payable attribute
            {
                handler.emit_err(CompileError::TraitImplPayabilityMismatch {
                    fn_name: impl_method.name.clone(),
                    interface_name: interface_name(),
                    missing_impl_attribute: true,
                    span: impl_method.span.clone(),
                });
            }
            (false, true) =>
            // implementation has extra payable attribute, not mentioned by signature
            {
                handler.emit_err(CompileError::TraitImplPayabilityMismatch {
                    fn_name: impl_method.name.clone(),
                    interface_name: interface_name(),
                    missing_impl_attribute: false,
                    span: impl_method.span.clone(),
                });
            }
            (true, true) | (false, false) => (), // no payability mismatch
        }

        // this subst is required to replace associated types, namely TypeInfo::TraitType.
        let mut impl_method_return_type_id = impl_method.return_type.type_id;
        impl_method_return_type_id.subst(&ctx.type_subst(), engines);

        let mut impl_method_signature_return_type_type_id =
            impl_method_signature.return_type.type_id;
        impl_method_signature_return_type_type_id.subst(&ctx.type_subst(), engines);

        if !type_engine.get(impl_method_return_type_id).eq(
            &type_engine.get(impl_method_signature_return_type_type_id),
            engines,
        ) {
            return Err(
                handler.emit_err(CompileError::MismatchedTypeInInterfaceSurface {
                    interface_name: interface_name(),
                    span: impl_method.return_type.span.clone(),
                    decl_type: "function".to_string(),
                    expected: engines
                        .help_out(impl_method_signature_return_type_type_id)
                        .to_string(),
                    given: engines.help_out(impl_method_return_type_id).to_string(),
                }),
            );
        }

        // We need to add impl type parameters to the method's type parameters
        // so that in-line monomorphization can complete.
        //
        // We also need to add impl type parameters to the method's type
        // parameters so the type constraints are correctly applied to the method.
        //
        // NOTE: this is a semi-hack that is used to force monomorphization of
        // trait methods that contain a generic defined in the parent impl...
        // without stuffing the generic into the method's type parameters, its
        // not currently possible to monomorphize on that generic at function
        // application time.
        impl_method.type_parameters.append(
            &mut impl_type_parameters
                .iter()
                .cloned()
                .map(|mut t| {
                    t.is_from_parent = true;
                    t
                })
                .collect::<Vec<_>>(),
        );

        Ok(impl_method)
    })
}

#[allow(clippy::too_many_arguments)]
fn type_check_const_decl(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    const_decl: &ConstantDeclaration,
    trait_name: &CallPath,
    is_contract: bool,
    impld_constant_ids: &ItemMap,
    constant_checklist: &BTreeMap<Ident, Arc<ty::TyConstantDecl>>,
) -> Result<ty::TyConstantDecl, ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    // Create a new type parameter for the "self type".
    let self_type_param = TypeParameter::new_self_type(engines, const_decl.span.clone());
    let self_type_id = self_type_param.type_id;

    let mut ctx = ctx
        .by_ref()
        .with_help_text("")
        .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None));

    let interface_name = || -> InterfaceName {
        if is_contract {
            InterfaceName::Abi(trait_name.suffix.clone())
        } else {
            InterfaceName::Trait(trait_name.suffix.clone())
        }
    };

    // type check the constant declaration
    let const_decl = ty::TyConstantDecl::type_check(handler, ctx.by_ref(), const_decl.clone())?;

    let const_name = const_decl.call_path.suffix.clone();

    // Ensure that there aren't multiple definitions of this constant
    if impld_constant_ids.contains_key(&(const_name.clone(), self_type_id)) {
        return Err(
            handler.emit_err(CompileError::MultipleDefinitionsOfConstant {
                name: const_name.clone(),
                span: const_name.span(),
            }),
        );
    }

    // Ensure that the constant checklist contains this constant.
    let const_decl_signature = match constant_checklist.get(&const_name) {
        Some(const_decl) => const_decl.clone(),
        None => {
            return Err(
                handler.emit_err(CompileError::ConstantNotAPartOfInterfaceSurface {
                    name: const_name.clone(),
                    interface_name: interface_name(),
                    span: const_name.span(),
                }),
            );
        }
    };

    // this subst is required to replace associated types, namely TypeInfo::TraitType.
    let mut const_decl_type_id = const_decl.type_ascription.type_id;
    const_decl_type_id.subst(&ctx.type_subst(), engines);

    let mut const_decl_signature_type_id = const_decl_signature.type_ascription.type_id;
    const_decl_signature_type_id.subst(&ctx.type_subst(), engines);

    // unify the types from the constant with the constant signature
    if !type_engine
        .get(const_decl_type_id)
        .eq(&type_engine.get(const_decl_signature_type_id), engines)
    {
        return Err(
            handler.emit_err(CompileError::MismatchedTypeInInterfaceSurface {
                interface_name: interface_name(),
                span: const_decl.span.clone(),
                decl_type: "constant".to_string(),
                given: engines.help_out(const_decl_type_id).to_string(),
                expected: engines.help_out(const_decl_signature_type_id).to_string(),
            }),
        );
    }

    Ok(const_decl)
}

#[allow(clippy::too_many_arguments)]
fn type_check_type_decl(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    type_decl: &TraitTypeDeclaration,
    trait_name: &CallPath,
    self_type: TypeId,
    is_contract: bool,
    impld_type_ids: &ItemMap,
    type_checklist: &BTreeMap<Ident, Arc<ty::TyTraitType>>,
) -> Result<ty::TyTraitType, ErrorEmitted> {
    let engines = ctx.engines();
    let type_engine = engines.te();

    let mut ctx = ctx
        .by_ref()
        .with_help_text("")
        .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None));

    let interface_name = || -> InterfaceName {
        if is_contract {
            InterfaceName::Abi(trait_name.suffix.clone())
        } else {
            InterfaceName::Trait(trait_name.suffix.clone())
        }
    };

    // type check the type declaration
    let type_decl = ty::TyTraitType::type_check(handler, ctx.by_ref(), type_decl.clone())?;

    let type_name = type_decl.name.clone();

    // Ensure that there aren't multiple definitions of this type
    if impld_type_ids.contains_key(&(type_name.clone(), self_type)) {
        return Err(handler.emit_err(CompileError::MultipleDefinitionsOfType {
            name: type_name.clone(),
            span: type_name.span(),
        }));
    }

    // Ensure that the type checklist contains this type.
    match type_checklist.get(&type_name) {
        Some(_type_decl) => {}
        None => {
            return Err(
                handler.emit_err(CompileError::TypeNotAPartOfInterfaceSurface {
                    name: type_name.clone(),
                    interface_name: interface_name(),
                    span: type_name.span(),
                }),
            );
        }
    };

    Ok(type_decl)
}

/// Given an array of [TypeParameter] `type_parameters`, checks to see if any of
/// the type parameters are unconstrained on the signature of the impl block.
///
/// An type parameter is unconstrained on the signature of the impl block when
/// it is not used in either the type arguments to the trait name or the type
/// arguments to the type the trait is implementing for.
///
/// Here is an example that would compile:
///
/// ```ignore
/// trait Test<T> {
///     fn test_it(self, the_value: T) -> T;
/// }
///
/// impl<T, F> Test<T> for FooBarData<F> {
///     fn test_it(self, the_value: T) -> T {
///         the_value
///     }
/// }
/// ```
///
/// Here is an example that would not compile, as the `T` is unconstrained:
///
/// ```ignore
/// trait Test {
///     fn test_it<G>(self, the_value: G) -> G;
/// }
///
/// impl<T, F> Test for FooBarData<F> {
///     fn test_it<G>(self, the_value: G) -> G {
///         the_value
///     }
/// }
/// ```
fn check_for_unconstrained_type_parameters(
    handler: &Handler,
    engines: &Engines,
    type_parameters: &[TypeParameter],
    trait_type_arguments: &[TypeArgument],
    self_type: TypeId,
) -> Result<(), ErrorEmitted> {
    // Create a list of defined generics, with the generic and a span.
    // Purposefully exclude the "self" type parameters.
    let mut defined_generics: HashMap<_, _> = HashMap::from_iter(
        type_parameters
            .iter()
            .map(|x| ((*engines.te().get(x.type_id)).clone(), x.span()))
            .map(|(thing, sp)| (WithEngines::new(thing, engines), sp)),
    );

    // create a list of the generics in use in the impl signature
    let mut generics_in_use = HashSet::new();
    for type_arg in trait_type_arguments.iter() {
        generics_in_use.extend(type_arg.type_id.extract_nested_generics(engines));
    }
    generics_in_use.extend(self_type.extract_nested_generics(engines));

    // TODO: add a lookup in the trait constraints here and add it to
    // generics_in_use

    // deduct the generics in use from the defined generics
    for generic in generics_in_use.into_iter() {
        defined_generics.remove(&generic);
    }

    handler.scope(|handler| {
        // create an error for all of the leftover generics
        for (k, v) in defined_generics.into_iter() {
            handler.emit_err(CompileError::UnconstrainedGenericParameter {
                ty: format!("{k}"),
                span: v,
            });
        }

        Ok(())
    })
}

fn handle_supertraits(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    self_type: TypeId,
    supertraits: &[Supertrait],
) -> Result<(InterfaceItemMap, ItemMap), ErrorEmitted> {
    let engines = ctx.engines;
    let decl_engine = engines.de();

    let mut interface_surface_item_ids: InterfaceItemMap = BTreeMap::new();
    let mut impld_item_refs: ItemMap = BTreeMap::new();

    handler.scope(|handler| {
        for supertrait in supertraits.iter() {
            // Right now we don't have the ability to support defining a supertrait
            // using a callpath directly, so we check to see if the user has done
            // this and we disallow it.
            if !supertrait.name.prefixes.is_empty() {
                handler.emit_err(CompileError::UnimplementedWithHelp(
                    "Using module paths to define supertraits is not supported yet.",
                    "try importing the trait with a \"use\" statement instead",
                    supertrait.span(),
                ));
                continue;
            }

            match ctx
                .namespace
                // Use the default Handler to avoid emitting the redundant SymbolNotFound error.
                .resolve_call_path(
                    &Handler::default(),
                    engines,
                    &supertrait.name,
                    ctx.self_type(),
                )
                .ok()
            {
                Some(ty::TyDecl::TraitDecl(ty::TraitDecl { decl_id, .. })) => {
                    let trait_decl = decl_engine.get_trait(&decl_id);

                    // Right now we don't parse type arguments for supertraits, so
                    // we should give this error message to users.
                    if !trait_decl.type_parameters.is_empty() {
                        handler.emit_err(CompileError::Unimplemented(
                            "Using generic traits as supertraits is not supported yet.",
                            supertrait.name.span(),
                        ));
                        continue;
                    }

                    // Retrieve the interface surface and implemented method ids for
                    // this trait.
                    let (trait_interface_surface_items_ids, trait_impld_item_refs) = trait_decl
                        .retrieve_interface_surface_and_implemented_items_for_type(
                            ctx.by_ref(),
                            self_type,
                            &supertrait.name,
                        );
                    interface_surface_item_ids.extend(trait_interface_surface_items_ids);
                    impld_item_refs.extend(trait_impld_item_refs);

                    // Retrieve the interface surfaces and implemented methods for
                    // the supertraits of this type.
                    let (next_interface_supertrait_decl_refs, next_these_supertrait_decl_refs) =
                        match handle_supertraits(
                            handler,
                            ctx.by_ref(),
                            self_type,
                            &trait_decl.supertraits,
                        ) {
                            Ok(res) => res,
                            Err(_) => continue,
                        };
                    interface_surface_item_ids.extend(next_interface_supertrait_decl_refs);
                    impld_item_refs.extend(next_these_supertrait_decl_refs);
                }
                Some(ty::TyDecl::AbiDecl { .. }) => {
                    // we allow ABIs as superABIs now
                }
                _ => {
                    handler.emit_err(CompileError::TraitNotFound {
                        name: supertrait.name.to_string(),
                        span: supertrait.name.span(),
                    });
                }
            }
        }

        Ok((interface_surface_item_ids, impld_item_refs))
    })
}

impl TypeCheckAnalysis for ty::ImplTrait {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        let decl_engine = ctx.engines.de();
        let impl_trait = decl_engine.get_impl_trait(&self.decl_id);

        // Lets create a graph node for the impl trait and for every item in the trait.
        ctx.push_nodes_for_impl_trait(self);

        // Now lets analyze each impl trait item.
        for (i, item) in impl_trait.items.iter().enumerate() {
            let _node = ctx.items_node_stack[i];
            item.type_check_analyze(handler, ctx)?;
        }

        // Clear the work-in-progress node stack.
        ctx.items_node_stack.clear();

        Ok(())
    }
}

impl TypeCheckFinalization for TyImplTrait {
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
