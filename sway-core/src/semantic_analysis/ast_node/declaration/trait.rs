use std::collections::{BTreeMap, HashSet};

use sway_error::{
    error::CompileError,
    warning::{CompileWarning, Warning},
};
use sway_types::{style::is_upper_camel_case, Ident, Spanned};

use crate::{
    decl_engine::*,
    error::*,
    language::{parsed::*, ty, CallPath},
    semantic_analysis::{declaration::insert_supertraits_into_namespace, Mode, TypeCheckContext},
    type_system::*,
};

impl ty::TyTraitDecl {
    pub(crate) fn type_check(
        ctx: TypeCheckContext,
        trait_decl: TraitDeclaration,
    ) -> CompileResult<(Self, SubstList)> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

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
            warnings.push(CompileWarning {
                span: name.span(),
                warning_content: Warning::NonClassCaseTraitName { name: name.clone() },
            })
        }

        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;
        let engines = ctx.engines();

        // A temporary namespace for checking within the trait's scope.
        let self_type = type_engine.insert(decl_engine, TypeInfo::SelfType);
        let mut trait_namespace = ctx.namespace.clone();
        let mut ctx = ctx.scoped(&mut trait_namespace).with_self_type(self_type);

        // Type check the type parameters. This will also insert them into the
        // current namespace.
        let (new_type_parameters, subst_list) = check!(
            TypeParameter::type_check_type_params(ctx.by_ref(), type_parameters, true),
            return err(warnings, errors),
            warnings,
            errors
        );
        ctx.namespace
            .subst_list_stack_mut()
            .push(subst_list.clone());

        // Recursively make the interface surfaces and methods of the
        // supertraits available to this trait.
        check!(
            insert_supertraits_into_namespace(ctx.by_ref(), self_type, &supertraits),
            return err(warnings, errors),
            warnings,
            errors
        );

        // type check the interface surface
        let mut new_interface_surface = vec![];
        let mut dummy_interface_surface = vec![];

        let mut ids: HashSet<Ident> = HashSet::default();

        for item in interface_surface.into_iter() {
            let decl_name = match item {
                TraitItem::TraitFn(method) => {
                    let (method, subst_list) = check!(
                        ty::TyTraitFn::type_check(ctx.by_ref(), method),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    let decl_ref = decl_engine.insert(method.clone(), subst_list.clone());
                    dummy_interface_surface.push(ty::TyImplItem::Fn(
                        decl_engine.insert(method.to_dummy_func(Mode::NonAbi), subst_list),
                    ));
                    new_interface_surface.push(ty::TyTraitInterfaceItem::TraitFn(decl_ref));
                    method.name.clone()
                }
                TraitItem::Constant(const_decl) => {
                    let const_decl = check!(
                        ty::TyConstantDecl::type_check(ctx.by_ref(), const_decl.clone(),),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    let decl_ref = ctx.decl_engine.insert(const_decl.clone(), SubstList::new());
                    new_interface_surface
                        .push(ty::TyTraitInterfaceItem::Constant(decl_ref.clone()));

                    let const_name = const_decl.call_path.suffix.clone();
                    check!(
                        ctx.namespace.insert_symbol(
                            const_name.clone(),
                            ty::TyDecl::ConstantDecl {
                                name: const_name.clone(),
                                decl_id: *decl_ref.id(),
                                decl_span: const_decl.span.clone()
                            }
                        ),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );

                    const_name
                }
            };

            if !ids.insert(decl_name.clone()) {
                errors.push(CompileError::MultipleDefinitionsOfName {
                    name: decl_name.clone(),
                    span: decl_name.span(),
                })
            }
        }

        // insert placeholder functions representing the interface surface
        // to allow methods to use those functions
        check!(
            ctx.namespace.insert_trait_implementation(
                CallPath {
                    prefixes: vec![],
                    suffix: name.clone(),
                    is_absolute: false,
                },
                new_type_parameters.iter().map(|x| x.into()).collect(),
                self_type,
                &dummy_interface_surface,
                &span,
                false,
                engines,
            ),
            return err(warnings, errors),
            warnings,
            errors
        );

        // type check the items
        let mut new_items = vec![];
        for method in methods.into_iter() {
            let (method, subst_list) = check!(
                ty::TyFunctionDecl::type_check(ctx.by_ref(), method.clone(), true, false),
                ty::TyFunctionDecl::error(method),
                warnings,
                errors
            );
            new_items.push(ty::TyTraitItem::Fn(decl_engine.insert(method, subst_list)));
        }

        let typed_trait_decl = ty::TyTraitDecl {
            name,
            type_parameters: new_type_parameters,
            interface_surface: new_interface_surface,
            items: new_items,
            supertraits,
            visibility,
            attributes,
            span,
        };
        ok((typed_trait_decl, subst_list), warnings, errors)
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
                    interface_surface_item_refs.insert(decl_ref.name().clone(), item.clone());
                }
                ty::TyTraitInterfaceItem::Constant(decl_ref) => {
                    interface_surface_item_refs.insert(decl_ref.name().clone(), item.clone());
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
                    impld_item_refs.insert(decl_ref.name().clone(), item.clone());
                }
                ty::TyTraitItem::Constant(decl_ref) => {
                    impld_item_refs.insert(decl_ref.name().clone(), item.clone());
                }
            };
        }

        (interface_surface_item_refs, impld_item_refs)
    }

    /// Retrieves the interface surface, items, and implemented items for
    /// this trait.
    pub(crate) fn retrieve_interface_surface_and_items_and_implemented_items_for_type(
        &self,
        ctx: TypeCheckContext,
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

        let decl_engine = ctx.decl_engine;
        let engines = ctx.engines();

        // Retrieve the interface surface for this trait.
        for item in interface_surface.iter() {
            match item {
                ty::TyTraitInterfaceItem::TraitFn(decl_ref) => {
                    interface_surface_item_refs.insert(decl_ref.name().clone(), item.clone());
                }
                ty::TyTraitInterfaceItem::Constant(decl_ref) => {
                    interface_surface_item_refs.insert(decl_ref.name().clone(), item.clone());
                }
            }
        }

        // Retrieve the trait items for this trait.
        for item in items.iter() {
            match item {
                ty::TyTraitItem::Fn(decl_ref) => {
                    item_refs.insert(decl_ref.name().clone(), item.clone());
                }
                ty::TyTraitItem::Constant(decl_ref) => {
                    item_refs.insert(decl_ref.name().clone(), item.clone());
                }
            }
        }

        // Retrieve the implemented items for this type.
        todo!();
        // let type_mapping = TypeSubstMap::from_type_parameters_and_type_arguments(
        //     type_parameters
        //         .iter()
        //         .map(|type_param| type_param.type_id)
        //         .collect(),
        //     type_arguments
        //         .iter()
        //         .map(|type_arg| type_arg.type_id)
        //         .collect(),
        // );
        // for item in ctx
        //     .namespace
        //     .get_items_for_type_and_trait_name(engines, type_id, call_path)
        //     .into_iter()
        // {
        //     match item {
        //         ty::TyTraitItem::Fn(mut decl_ref) => {
        //             decl_ref.subst(&type_mapping, engines);
        //             impld_item_refs.insert(decl_ref.name().clone(), TyTraitItem::Fn(decl_ref));
        //         }
        //         ty::TyTraitItem::Constant(mut decl_ref) => {
        //             decl_ref.subst(&type_mapping, engines);
        //             impld_item_refs
        //                 .insert(decl_ref.name().clone(), TyTraitItem::Constant(decl_ref));
        //         }
        //     }
        // }

        (interface_surface_item_refs, item_refs, impld_item_refs)
    }

    pub(crate) fn insert_interface_surface_and_items_into_namespace(
        &self,
        ctx: TypeCheckContext,
        trait_name: &CallPath,
        type_arguments: &[TypeArgument],
        type_id: TypeId,
    ) {
        let decl_engine = ctx.decl_engine;
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
        todo!();
        // let type_mapping = TypeSubstMap::from_type_parameters_and_type_arguments(
        //     type_parameters
        //         .iter()
        //         .map(|type_param| type_param.type_id)
        //         .collect(),
        //     type_arguments
        //         .iter()
        //         .map(|type_arg| type_arg.type_id)
        //         .collect(),
        // );
        // for item in interface_surface.iter() {
        //     match item {
        //         ty::TyTraitInterfaceItem::TraitFn(decl_ref) => {
        //             let mut decl_ref = decl_ref.clone();
        //             decl_ref.subst(&type_mapping, engines);
        //             todo!();
        //             // all_items.push(TyImplItem::Fn(
        //             //     ctx.decl_engine
        //             //         .insert(method.to_dummy_func(Mode::NonAbi))
        //             //         .with_parent(ctx.decl_engine, (*decl_ref.id()).into()),
        //             // ));
        //         }
        //         ty::TyTraitInterfaceItem::Constant(decl_ref) => {
        //             let mut decl_ref = decl_ref.clone();
        //             decl_ref.subst(&type_mapping, engines);
        //             ctx.namespace.insert_symbol(
        //                 decl_ref.name().clone(),
        //                 ty::TyDeclaration::ConstantDeclaration {
        //                     name: decl_ref.name().clone(),
        //                     decl_id: *decl_ref.id(),
        //                     decl_span: decl_ref.decl_span().clone(),
        //                 },
        //             );
        //         }
        //     }
        // }
        // for item in items.iter() {
        //     match item {
        //         ty::TyTraitItem::Fn(decl_ref) => {
        //             let mut decl_ref = decl_ref.clone();
        //             decl_ref.subst(&type_mapping, engines);
        //             all_items.push(TyImplItem::Fn(decl_ref));
        //         }
        //         ty::TyTraitItem::Constant(decl_ref) => {
        //             let mut decl_ref = decl_ref.clone();
        //             decl_ref.subst(&type_mapping, engines);
        //             all_items.push(TyImplItem::Constant(decl_ref));
        //         }
        //     }
        // }

        // Insert the methods of the trait into the namespace.
        // Specifically do not check for conflicting definitions because
        // this is just a temporary namespace for type checking and
        // these are not actual impl blocks.
        ctx.namespace.insert_trait_implementation(
            trait_name.clone(),
            type_arguments.to_vec(),
            type_id,
            &all_items,
            &trait_name.span(),
            false,
            engines,
        );
    }
}
