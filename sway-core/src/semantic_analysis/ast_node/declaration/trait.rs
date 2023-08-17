use std::collections::{BTreeMap, HashSet};

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
    warning::{CompileWarning, Warning},
};
use sway_types::{style::is_upper_camel_case, Ident, Spanned};

use crate::{
    decl_engine::*,
    language::{
        parsed::*,
        ty::{self, TyImplItem, TyTraitItem},
        CallPath,
    },
    semantic_analysis::{
        declaration::{insert_supertraits_into_namespace, SupertraitOf},
        AbiMode, TypeCheckContext,
    },
    type_system::*,
};

impl ty::TyTraitDecl {
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

        let type_engine = ctx.engines.te();
        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();

        // A temporary namespace for checking within the trait's scope.
        let mut trait_namespace = ctx.namespace.clone();
        let mut ctx = ctx.scoped(&mut trait_namespace);

        // Create a new type parameter for the "self type".
        let self_type_param = TypeParameter::new_self_type(engines, name.span());
        let self_type = self_type_param.type_id;

        // Type check the type parameters.
        let new_type_parameters =
            TypeParameter::type_check_type_params(handler, ctx.by_ref(), type_parameters, Some(self_type_param.clone()))?;

        // Insert them into the current namespace.
        for p in &new_type_parameters {
            p.insert_into_namespace(handler, ctx.by_ref())?;
        }

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

        for item in interface_surface.into_iter() {
            let decl_name = match item {
                TraitItem::TraitFn(method) => {
                    let method = ty::TyTraitFn::type_check(handler, ctx.by_ref(), method)?;
                    let decl_ref = decl_engine.insert(method.clone());
                    dummy_interface_surface.push(ty::TyImplItem::Fn(
                        decl_engine
                            .insert(method.to_dummy_func(AbiMode::NonAbi))
                            .with_parent(decl_engine, (*decl_ref.id()).into()),
                    ));
                    new_interface_surface.push(ty::TyTraitInterfaceItem::TraitFn(decl_ref));
                    method.name.clone()
                }
                TraitItem::Constant(const_decl) => {
                    let const_decl =
                        ty::TyConstantDecl::type_check(handler, ctx.by_ref(), const_decl.clone())?;
                    let decl_ref = ctx.engines.de().insert(const_decl.clone());
                    new_interface_surface
                        .push(ty::TyTraitInterfaceItem::Constant(decl_ref.clone()));

                    let const_name = const_decl.call_path.suffix.clone();
                    ctx.insert_symbol(
                        handler,
                        const_name.clone(),
                        ty::TyDecl::ConstantDecl(ty::ConstantDecl {
                            name: const_name.clone(),
                            decl_id: *decl_ref.id(),
                            decl_span: const_decl.span.clone(),
                        }),
                    )?;

                    const_name
                }
                TraitItem::Error(_, _) => {
                    continue;
                }
            };

            if !ids.insert(decl_name.clone()) {
                handler.emit_err(CompileError::MultipleDefinitionsOfName {
                    name: decl_name.clone(),
                    span: decl_name.span(),
                });
            }
        }

        // insert placeholder functions representing the interface surface
        // to allow methods to use those functions
        ctx.namespace.insert_trait_implementation(
            handler,
            CallPath {
                prefixes: vec![],
                suffix: name.clone(),
                is_absolute: false,
            },
            new_type_parameters.iter().map(|x| x.into()).collect(),
            self_type,
            &dummy_interface_surface,
            &span,
            None,
            false,
            engines,
        )?;

        // Type check the items.
        let mut new_items = vec![];
        for method in methods.into_iter() {
            let method =
                ty::TyFunctionDecl::type_check(handler, ctx.by_ref(), method.clone(), true, false)
                    .unwrap_or_else(|_| ty::TyFunctionDecl::error(method));
            new_items.push(ty::TyTraitItem::Fn(decl_engine.insert(method)));
        }

        let typed_trait_decl = ty::TyTraitDecl {
            name,
            type_parameters: new_type_parameters,
            self_type: self_type_param,
            interface_surface: new_interface_surface,
            items: new_items,
            supertraits,
            visibility,
            attributes,
            span,
        };
        Ok(typed_trait_decl)
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
            }
        }

        // Retrieve the implemented items for this type.
        for item in ctx
            .namespace
            .get_items_for_type_and_trait_name(engines, type_id, call_path)
            .into_iter()
        {
            match &item {
                ty::TyTraitItem::Fn(decl_ref) => {
                    impld_item_refs.insert((decl_ref.name().clone(), type_id), item.clone());
                }
                ty::TyTraitItem::Constant(decl_ref) => {
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
        type_arguments: &[TypeArgument],
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
            }
        }

        // Retrieve the implemented items for this type.
        let type_mapping = TypeSubstMap::from_type_parameters_and_type_arguments(
            type_parameters
                .iter()
                .map(|type_param| type_param.type_id)
                .collect(),
            type_arguments
                .iter()
                .map(|type_arg| type_arg.type_id)
                .collect(),
        );
        for item in ctx
            .namespace
            .get_items_for_type_and_trait_name(engines, type_id, call_path)
            .into_iter()
        {
            match item {
                ty::TyTraitItem::Fn(decl_ref) => {
                    let mut method = decl_engine.get_function(&decl_ref);
                    method.subst(&type_mapping, engines);
                    impld_item_refs.insert(
                        (method.name.clone(), type_id),
                        TyTraitItem::Fn(
                            decl_engine
                                .insert(method)
                                .with_parent(decl_engine, (*decl_ref.id()).into()),
                        ),
                    );
                }
                ty::TyTraitItem::Constant(decl_ref) => {
                    let mut const_decl = decl_engine.get_constant(&decl_ref);
                    const_decl.subst(&type_mapping, engines);
                    impld_item_refs.insert(
                        (const_decl.call_path.suffix.clone(), type_id),
                        TyTraitItem::Constant(decl_engine.insert(const_decl)),
                    );
                }
            }
        }

        (interface_surface_item_refs, item_refs, impld_item_refs)
    }

    pub(crate) fn insert_interface_surface_and_items_into_namespace(
        &self,
        handler: &Handler,
        ctx: TypeCheckContext,
        trait_name: &CallPath,
        type_arguments: &[TypeArgument],
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
                .map(|type_param| type_param.type_id)
                .collect(),
            type_arguments
                .iter()
                .map(|type_arg| type_arg.type_id)
                .collect(),
        );

        for item in interface_surface.iter() {
            match item {
                ty::TyTraitInterfaceItem::TraitFn(decl_ref) => {
                    let mut method = decl_engine.get_trait_fn(decl_ref);
                    method.subst(&type_mapping, engines);
                    all_items.push(TyImplItem::Fn(
                        ctx.engines
                            .de()
                            .insert(method.to_dummy_func(AbiMode::NonAbi))
                            .with_parent(ctx.engines.de(), (*decl_ref.id()).into()),
                    ));
                }
                ty::TyTraitInterfaceItem::Constant(decl_ref) => {
                    let const_decl = decl_engine.get_constant(decl_ref);
                    let const_name = const_decl.call_path.suffix.clone();
                    all_items.push(TyImplItem::Constant(decl_ref.clone()));
                    let const_shadowing_mode = ctx.const_shadowing_mode();
                    let _ = ctx.namespace.insert_symbol(
                        handler,
                        const_name.clone(),
                        ty::TyDecl::ConstantDecl(ty::ConstantDecl {
                            name: const_name,
                            decl_id: *decl_ref.id(),
                            decl_span: const_decl.span.clone(),
                        }),
                        const_shadowing_mode,
                    );
                }
            }
        }
        for item in items.iter() {
            match item {
                ty::TyTraitItem::Fn(decl_ref) => {
                    let mut method = decl_engine.get_function(decl_ref);
                    method.subst(&type_mapping, engines);
                    all_items.push(TyImplItem::Fn(
                        ctx.engines
                            .de()
                            .insert(method)
                            .with_parent(ctx.engines.de(), (*decl_ref.id()).into()),
                    ));
                }
                ty::TyTraitItem::Constant(decl_ref) => {
                    let mut const_decl = decl_engine.get_constant(decl_ref);
                    const_decl.subst(&type_mapping, engines);
                    all_items.push(TyImplItem::Constant(ctx.engines.de().insert(const_decl)));
                }
            }
        }

        // Insert the methods of the trait into the namespace.
        // Specifically do not check for conflicting definitions because
        // this is just a temporary namespace for type checking and
        // these are not actual impl blocks.
        let _ = ctx.namespace.insert_trait_implementation(
            &Handler::default(),
            trait_name.clone(),
            type_arguments.to_vec(),
            type_id,
            &all_items,
            &trait_name.span(),
            Some(self.span()),
            false,
            engines,
        );
    }
}
