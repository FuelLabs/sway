use std::collections::{BTreeMap, HashMap, HashSet};

use sway_error::error::{CompileError, InterfaceName};
use sway_types::{Ident, Span, Spanned};

use crate::{
    decl_engine::*,
    engine_threading::*,
    error::*,
    language::{
        parsed::*,
        ty::{self, TyImplItem, TyTraitInterfaceItem, TyTraitItem},
        *,
    },
    semantic_analysis::{Mode, TypeCheckContext},
    type_system::*,
};

impl ty::TyImplTrait {
    pub(crate) fn type_check_impl_trait(
        mut ctx: TypeCheckContext,
        impl_trait: ImplTrait,
    ) -> CompileResult<Self> {
        let mut errors = vec![];
        let mut warnings = vec![];

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

        // create a namespace for the impl
        let mut impl_namespace = ctx.namespace.clone();
        let mut ctx = ctx.by_ref().scoped(&mut impl_namespace).allow_functions();

        // Type check the type parameters. This will also insert them into the
        // current namespace.
        let new_impl_type_parameters = check!(
            TypeParameter::type_check_type_params(ctx.by_ref(), impl_type_parameters),
            return err(warnings, errors),
            warnings,
            errors
        );

        // resolve the types of the trait type arguments
        for type_arg in trait_type_arguments.iter_mut() {
            type_arg.type_id = check!(
                ctx.resolve_type_without_self(type_arg.type_id, &type_arg.span, None),
                return err(warnings, errors),
                warnings,
                errors
            );
        }

        // type check the type that we are implementing for

        implementing_for.type_id = check!(
            ctx.resolve_type_without_self(implementing_for.type_id, &implementing_for.span, None),
            return err(warnings, errors),
            warnings,
            errors
        );

        // check to see if this type is supported in impl blocks
        check!(
            type_engine
                .get(implementing_for.type_id)
                .expect_is_supported_in_impl_blocks_self(&implementing_for.span),
            return err(warnings, errors),
            warnings,
            errors
        );

        // check for unconstrained type parameters
        check!(
            check_for_unconstrained_type_parameters(
                engines,
                &new_impl_type_parameters,
                &trait_type_arguments,
                implementing_for.type_id,
            ),
            return err(warnings, errors),
            warnings,
            errors
        );

        // Update the context with the new `self` type.
        let mut ctx = ctx
            .with_self_type(implementing_for.type_id)
            .with_help_text("")
            .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown));

        let impl_trait = match ctx
            .namespace
            .resolve_call_path(&trait_name)
            .ok(&mut warnings, &mut errors)
            .cloned()
        {
            Some(ty::TyDecl::TraitDecl(ty::TraitDecl { decl_id, .. })) => {
                let mut trait_decl = decl_engine.get_trait(&decl_id);

                // monomorphize the trait declaration
                check!(
                    ctx.monomorphize(
                        &mut trait_decl,
                        &mut trait_type_arguments,
                        EnforceTypeArguments::Yes,
                        &trait_name.span()
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                // Insert the interface surface and methods from this trait into
                // the namespace.
                trait_decl.insert_interface_surface_and_items_into_namespace(
                    ctx.by_ref(),
                    &trait_name,
                    &trait_type_arguments,
                    implementing_for.type_id,
                );

                let new_items = check!(
                    type_check_trait_implementation(
                        ctx.by_ref(),
                        &new_impl_type_parameters,
                        &trait_decl.type_parameters,
                        &trait_type_arguments,
                        &trait_decl.supertraits,
                        &trait_decl.interface_surface,
                        &trait_decl.items,
                        &items,
                        &trait_name,
                        &block_span,
                        false,
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
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
                    errors.push(CompileError::ImplAbiForNonContract {
                        span: implementing_for.span(),
                        ty: engines.help_out(implementing_for.type_id).to_string(),
                    });
                }

                let mut ctx = ctx.with_mode(Mode::ImplAbiFn);

                // Insert the interface surface and methods from this trait into
                // the namespace.
                abi.insert_interface_surface_and_items_into_namespace(
                    ctx.by_ref(),
                    implementing_for.type_id,
                );

                let new_items = check!(
                    type_check_trait_implementation(
                        ctx.by_ref(),
                        &[], // this is empty because abi definitions don't support generics,
                        &[], // this is empty because abi definitions don't support generics,
                        &[], // this is empty because abi definitions don't support generics,
                        &abi.supertraits,
                        &abi.interface_surface,
                        &abi.items,
                        &items,
                        &trait_name,
                        &block_span,
                        true
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                ty::TyImplTrait {
                    impl_type_parameters: vec![], // this is empty because abi definitions don't support generics
                    trait_name,
                    trait_type_arguments: vec![], // this is empty because abi definitions don't support generics
                    trait_decl_ref: Some(DeclRef::new(abi.name.clone(), decl_id.into(), abi.span)),
                    span: block_span,
                    items: new_items,
                    implementing_for,
                }
            }
            Some(_) | None => {
                errors.push(CompileError::UnknownTrait {
                    name: trait_name.suffix.clone(),
                    span: trait_name.span(),
                });
                return err(warnings, errors);
            }
        };
        ok(impl_trait, warnings, errors)
    }

    pub(crate) fn type_check_impl_self(
        ctx: TypeCheckContext,
        impl_self: ImplSelf,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];

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
        let mut ctx = ctx.scoped(&mut impl_namespace).allow_functions();

        // create the trait name
        let trait_name = CallPath {
            prefixes: vec![],
            suffix: match &type_engine.get(implementing_for.type_id) {
                TypeInfo::Custom { call_path, .. } => call_path.suffix.clone(),
                _ => Ident::new_with_override("r#Self".into(), implementing_for.span()),
            },
            is_absolute: false,
        };

        // Type check the type parameters. This will also insert them into the
        // current namespace.
        let new_impl_type_parameters = check!(
            TypeParameter::type_check_type_params(ctx.by_ref(), impl_type_parameters),
            return err(warnings, errors),
            warnings,
            errors
        );

        // type check the type that we are implementing for
        implementing_for.type_id = check!(
            ctx.resolve_type_without_self(implementing_for.type_id, &implementing_for.span, None),
            return err(warnings, errors),
            warnings,
            errors
        );

        // check to see if this type is supported in impl blocks
        check!(
            type_engine
                .get(implementing_for.type_id)
                .expect_is_supported_in_impl_blocks_self(&implementing_for.span),
            return err(warnings, errors),
            warnings,
            errors
        );

        // check for unconstrained type parameters
        check!(
            check_for_unconstrained_type_parameters(
                engines,
                &new_impl_type_parameters,
                &[],
                implementing_for.type_id,
            ),
            return err(warnings, errors),
            warnings,
            errors
        );

        check!(
            implementing_for.type_id.check_type_parameter_bounds(
                &ctx,
                &implementing_for.span,
                vec![]
            ),
            return err(warnings, errors),
            warnings,
            errors
        );

        let mut ctx = ctx
            .with_self_type(implementing_for.type_id)
            .with_help_text("")
            .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown));

        // Insert implementing type decl as `Self` symbol.
        let self_decl: Option<ty::TyDecl> = match type_engine.get(implementing_for.type_id) {
            TypeInfo::Enum(r) => Some(r.into()),
            TypeInfo::Struct(r) => Some(r.into()),
            _ => None,
        };
        if let Some(self_decl) = self_decl {
            ctx.namespace
                .insert_symbol(Ident::new_no_span("Self".to_string()), self_decl);
        }

        // type check the items inside of the impl block
        let mut new_items = vec![];

        for item in items.into_iter() {
            match item {
                ImplItem::Fn(fn_decl) => {
                    let fn_decl = check!(
                        ty::TyFunctionDecl::type_check(ctx.by_ref(), fn_decl, true, true),
                        continue,
                        warnings,
                        errors
                    );
                    new_items.push(TyImplItem::Fn(decl_engine.insert(fn_decl)));
                }
                ImplItem::Constant(const_decl) => {
                    let const_decl = check!(
                        ty::TyConstantDecl::type_check(ctx.by_ref(), const_decl),
                        continue,
                        warnings,
                        errors
                    );
                    let decl_ref = decl_engine.insert(const_decl);
                    new_items.push(TyImplItem::Constant(decl_ref.clone()));

                    check!(
                        ctx.namespace.insert_symbol(
                            decl_ref.name().clone(),
                            ty::TyDecl::ConstantDecl(ty::ConstantDecl {
                                name: decl_ref.name().clone(),
                                decl_id: *decl_ref.id(),
                                decl_span: decl_ref.span().clone()
                            })
                        ),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                }
            }
        }
        if !errors.is_empty() {
            return err(warnings, errors);
        }

        let impl_trait = ty::TyImplTrait {
            impl_type_parameters: new_impl_type_parameters,
            trait_name,
            trait_type_arguments: vec![], // this is empty because impl selfs don't support generics on the "Self" trait,
            trait_decl_ref: None,
            span: block_span,
            items: new_items,
            implementing_for,
        };
        ok(impl_trait, warnings, errors)
    }
}

#[allow(clippy::too_many_arguments)]
fn type_check_trait_implementation(
    mut ctx: TypeCheckContext,
    impl_type_parameters: &[TypeParameter],
    trait_type_parameters: &[TypeParameter],
    trait_type_arguments: &[TypeArgument],
    trait_supertraits: &[Supertrait],
    trait_interface_surface: &[TyTraitInterfaceItem],
    trait_items: &[TyImplItem],
    impl_items: &[ImplItem],
    trait_name: &CallPath,
    block_span: &Span,
    is_contract: bool,
) -> CompileResult<Vec<TyImplItem>> {
    let mut errors = vec![];
    let mut warnings = vec![];

    let decl_engine = ctx.engines.de();
    let engines = ctx.engines();
    let self_type = ctx.self_type();

    // Check to see if the type that we are implementing for implements the
    // supertraits of this trait.
    check!(
        ctx.namespace
            .implemented_traits
            .check_if_trait_constraints_are_satisfied_for_type(
                self_type,
                &trait_supertraits
                    .iter()
                    .map(|x| x.into())
                    .collect::<Vec<_>>(),
                block_span,
                engines,
            ),
        return err(warnings, errors),
        warnings,
        errors
    );

    for (type_arg, type_param) in trait_type_arguments.iter().zip(trait_type_parameters) {
        check!(
            type_arg.type_id.check_type_parameter_bounds(
                &ctx,
                &type_arg.span(),
                type_param.trait_constraints.clone()
            ),
            return err(warnings, errors),
            warnings,
            errors
        );
    }

    // This map keeps track of the remaining functions in the interface surface
    // that still need to be implemented for the trait to be fully implemented.
    let mut method_checklist: BTreeMap<Ident, ty::TyTraitFn> = BTreeMap::new();

    // This map keeps track of the remaining constants in the interface surface
    // that still need to be implemented for the trait to be fully implemented.
    let mut constant_checklist: BTreeMap<Ident, ty::TyConstantDecl> = BTreeMap::new();

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
        let (this_supertrait_stub_method_refs, this_supertrait_impld_method_refs) = check!(
            handle_supertraits(ctx.by_ref(), trait_supertraits),
            return err(warnings, errors),
            warnings,
            errors
        );

        ctx.namespace.insert_trait_implementation(
            trait_name.clone(),
            trait_type_arguments.to_vec(),
            self_type,
            &this_supertrait_impld_method_refs
                .values()
                .cloned()
                .collect::<Vec<_>>(),
            &trait_name.span(),
            false,
            engines,
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
                interface_item_refs.insert(name, item.clone());
            }
            TyTraitInterfaceItem::Constant(decl_ref) => {
                let constant = decl_engine.get_constant(decl_ref);
                let name = constant.call_path.suffix.clone();
                constant_checklist.insert(name.clone(), constant);
                interface_item_refs.insert(name, item.clone());
            }
        }
    }

    for item in impl_items {
        match item {
            ImplItem::Fn(impl_method) => {
                let impl_method = check!(
                    type_check_impl_method(
                        ctx.by_ref(),
                        impl_type_parameters,
                        impl_method,
                        trait_name,
                        is_contract,
                        &impld_item_refs,
                        &method_checklist
                    ),
                    ty::TyFunctionDecl::error(impl_method.clone()),
                    warnings,
                    errors
                );

                // Remove this method from the checklist.
                let name = impl_method.name.clone();
                method_checklist.remove(&name);

                // Add this method to the "impld items".
                let decl_ref = decl_engine.insert(impl_method);
                impld_item_refs.insert(name, TyTraitItem::Fn(decl_ref));
            }
            ImplItem::Constant(const_decl) => {
                let const_decl = check!(
                    type_check_const_decl(
                        ctx.by_ref(),
                        const_decl,
                        trait_name,
                        is_contract,
                        &impld_item_refs,
                        &constant_checklist
                    ),
                    ty::TyConstantDecl::error(ctx.engines(), const_decl.clone()),
                    warnings,
                    errors
                );

                // Remove this constant from the checklist.
                let name = const_decl.call_path.suffix.clone();
                constant_checklist.remove(&name);

                // Add this constant to the "impld decls".
                let decl_ref = decl_engine.insert(const_decl);
                impld_item_refs.insert(name, TyTraitItem::Constant(decl_ref));
            }
        }
    }

    let mut all_items_refs: Vec<TyImplItem> = impld_item_refs.values().cloned().collect();

    // Retrieve the methods defined on the trait declaration and transform
    // them into the correct typing for this impl block by using the type
    // parameters from the original trait declaration and the type arguments of
    // the trait name in the current impl block that we are type checking and
    // using the stub decl ids from the interface surface and the new
    // decl ids from the newly implemented methods.
    let type_mapping = TypeSubstMap::from_type_parameters_and_type_arguments(
        trait_type_parameters
            .iter()
            .map(|type_param| type_param.type_id)
            .collect(),
        trait_type_arguments
            .iter()
            .map(|type_arg| type_arg.type_id)
            .collect(),
    );
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
                let mut method = decl_engine.get_function(decl_ref);
                method.replace_decls(&decl_mapping, engines);
                method.subst(&type_mapping, engines);
                method.replace_self_type(engines, ctx.self_type());
                all_items_refs.push(TyImplItem::Fn(
                    decl_engine
                        .insert(method)
                        .with_parent(decl_engine, (*decl_ref.id()).into()),
                ));
            }
            TyImplItem::Constant(decl_ref) => {
                let mut const_decl = decl_engine.get_constant(decl_ref);
                const_decl.replace_decls(&decl_mapping, engines);
                const_decl.subst(&type_mapping, engines);
                const_decl.replace_self_type(engines, ctx.self_type());
                all_items_refs.push(TyImplItem::Constant(decl_engine.insert(const_decl)));
            }
        }
    }

    // check that the implementation checklist is complete
    if !method_checklist.is_empty() {
        errors.push(CompileError::MissingInterfaceSurfaceMethods {
            span: block_span.clone(),
            missing_functions: method_checklist
                .into_keys()
                .map(|ident| ident.as_str().to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        });
    }

    if !constant_checklist.is_empty() {
        errors.push(CompileError::MissingInterfaceSurfaceConstants {
            span: block_span.clone(),
            missing_constants: constant_checklist
                .into_keys()
                .map(|ident| ident.as_str().to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        });
    }

    if errors.is_empty() {
        ok(all_items_refs, warnings, errors)
    } else {
        err(warnings, errors)
    }
}

fn type_check_impl_method(
    mut ctx: TypeCheckContext,
    impl_type_parameters: &[TypeParameter],
    impl_method: &FunctionDeclaration,
    trait_name: &CallPath,
    is_contract: bool,
    impld_item_refs: &ItemMap,
    method_checklist: &BTreeMap<Ident, ty::TyTraitFn>,
) -> CompileResult<ty::TyFunctionDecl> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let type_engine = ctx.engines.te();
    let engines = ctx.engines();
    let self_type = ctx.self_type();

    let mut ctx = ctx
        .by_ref()
        .with_help_text("")
        .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown));

    let interface_name = || -> InterfaceName {
        if is_contract {
            InterfaceName::Abi(trait_name.suffix.clone())
        } else {
            InterfaceName::Trait(trait_name.suffix.clone())
        }
    };

    // type check the function declaration
    let mut impl_method = check!(
        ty::TyFunctionDecl::type_check(ctx.by_ref(), impl_method.clone(), true, false),
        return err(warnings, errors),
        warnings,
        errors
    );

    // Ensure that there aren't multiple definitions of this function impl'd
    if impld_item_refs.contains_key(&impl_method.name.clone()) {
        errors.push(CompileError::MultipleDefinitionsOfFunction {
            name: impl_method.name.clone(),
            span: impl_method.name.span(),
        });
        return err(warnings, errors);
    }

    // Ensure that the method checklist contains this function.
    let mut impl_method_signature = match method_checklist.get(&impl_method.name) {
        Some(trait_fn) => trait_fn.clone(),
        None => {
            errors.push(CompileError::FunctionNotAPartOfInterfaceSurface {
                name: impl_method.name.clone(),
                interface_name: interface_name(),
                span: impl_method.name.span(),
            });
            return err(warnings, errors);
        }
    };

    // replace instances of `TypeInfo::SelfType` with a fresh
    // `TypeInfo::SelfType` to avoid replacing types in the stub trait
    // declaration
    impl_method_signature.replace_self_type(engines, self_type);

    // ensure this fn decl's parameters and signature lines up with the one
    // in the trait
    if impl_method.parameters.len() != impl_method_signature.parameters.len() {
        errors.push(
            CompileError::IncorrectNumberOfInterfaceSurfaceFunctionParameters {
                span: impl_method.parameters_span(),
                fn_name: impl_method.name.clone(),
                interface_name: interface_name(),
                num_parameters: impl_method_signature.parameters.len(),
                provided_parameters: impl_method.parameters.len(),
            },
        );
        return err(warnings, errors);
    }

    // unify the types from the parameters of the function declaration
    // with the parameters of the function signature
    for (impl_method_signature_param, impl_method_param) in impl_method_signature
        .parameters
        .iter_mut()
        .zip(&mut impl_method.parameters)
    {
        // TODO use trait constraints as part of the type here to
        // implement trait constraint solver */
        // Check if we have a non-ref mutable argument. That's not allowed.
        if impl_method_signature_param.is_mutable && !impl_method_signature_param.is_reference {
            errors.push(CompileError::MutableParameterNotSupported {
                param_name: impl_method_signature.name.clone(),
                span: impl_method_signature.name.span(),
            });
        }

        // check if reference / mutability of the parameters is incompatible
        if impl_method_param.is_mutable != impl_method_signature_param.is_mutable
            || impl_method_param.is_reference != impl_method_signature_param.is_reference
        {
            errors.push(CompileError::ParameterRefMutabilityMismatch {
                span: impl_method_param.mutability_span.clone(),
            });
        }

        if !type_engine.get(impl_method_param.type_argument.type_id).eq(
            &type_engine.get(impl_method_signature_param.type_argument.type_id),
            engines,
        ) {
            errors.push(CompileError::MismatchedTypeInInterfaceSurface {
                interface_name: interface_name(),
                span: impl_method_param.type_argument.span.clone(),
                decl_type: "function".to_string(),
                given: engines
                    .help_out(impl_method_param.type_argument.type_id)
                    .to_string(),
                expected: engines
                    .help_out(impl_method_signature_param.type_argument.type_id)
                    .to_string(),
            });
            continue;
        }
    }

    // check to see if the purity of the function declaration is the same
    // as the purity of the function signature
    if impl_method.purity != impl_method_signature.purity {
        errors.push(if impl_method_signature.purity == Purity::Pure {
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
            errors.push(CompileError::TraitImplPayabilityMismatch {
                fn_name: impl_method.name.clone(),
                interface_name: interface_name(),
                missing_impl_attribute: true,
                span: impl_method.span.clone(),
            });
        }
        (false, true) =>
        // implementation has extra payable attribute, not mentioned by signature
        {
            errors.push(CompileError::TraitImplPayabilityMismatch {
                fn_name: impl_method.name.clone(),
                interface_name: interface_name(),
                missing_impl_attribute: false,
                span: impl_method.span.clone(),
            });
        }
        (true, true) | (false, false) => (), // no payability mismatch
    }

    if !type_engine.get(impl_method.return_type.type_id).eq(
        &type_engine.get(impl_method_signature.return_type.type_id),
        engines,
    ) {
        errors.push(CompileError::MismatchedTypeInInterfaceSurface {
            interface_name: interface_name(),
            span: impl_method.return_type.span.clone(),
            decl_type: "function".to_string(),
            expected: engines
                .help_out(impl_method_signature.return_type)
                .to_string(),
            given: engines.help_out(impl_method.return_type).to_string(),
        });
        return err(warnings, errors);
    }

    // We need to add impl type parameters to the  method's type parameters
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

    if errors.is_empty() {
        ok(impl_method, warnings, errors)
    } else {
        err(warnings, errors)
    }
}

fn type_check_const_decl(
    mut ctx: TypeCheckContext,
    const_decl: &ConstantDeclaration,
    trait_name: &CallPath,
    is_contract: bool,
    impld_constant_ids: &ItemMap,
    constant_checklist: &BTreeMap<Ident, ty::TyConstantDecl>,
) -> CompileResult<ty::TyConstantDecl> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let type_engine = ctx.engines.te();
    let engines = ctx.engines();
    let self_type = ctx.self_type();

    let mut ctx = ctx
        .by_ref()
        .with_help_text("")
        .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown));

    let interface_name = || -> InterfaceName {
        if is_contract {
            InterfaceName::Abi(trait_name.suffix.clone())
        } else {
            InterfaceName::Trait(trait_name.suffix.clone())
        }
    };

    // type check the constant declaration
    let const_decl = check!(
        ty::TyConstantDecl::type_check(ctx.by_ref(), const_decl.clone()),
        return err(warnings, errors),
        warnings,
        errors
    );

    let const_name = const_decl.call_path.suffix.clone();

    // Ensure that there aren't multiple definitions of this constant
    if impld_constant_ids.contains_key(&const_name) {
        errors.push(CompileError::MultipleDefinitionsOfConstant {
            name: const_name.clone(),
            span: const_name.span(),
        });
        return err(warnings, errors);
    }

    // Ensure that the constant checklist contains this constant.
    let mut const_decl_signature = match constant_checklist.get(&const_name) {
        Some(const_decl) => const_decl.clone(),
        None => {
            errors.push(CompileError::ConstantNotAPartOfInterfaceSurface {
                name: const_name.clone(),
                interface_name: interface_name(),
                span: const_name.span(),
            });
            return err(warnings, errors);
        }
    };

    // replace instances of `TypeInfo::SelfType` with a fresh
    // `TypeInfo::SelfType` to avoid replacing types in the stub constant
    // declaration
    const_decl_signature.replace_self_type(engines, self_type);

    // unify the types from the constant with the constant signature
    if !type_engine.get(const_decl.type_ascription.type_id).eq(
        &type_engine.get(const_decl_signature.type_ascription.type_id),
        engines,
    ) {
        errors.push(CompileError::MismatchedTypeInInterfaceSurface {
            interface_name: interface_name(),
            span: const_decl.span.clone(),
            decl_type: "constant".to_string(),
            given: engines
                .help_out(const_decl.type_ascription.type_id)
                .to_string(),
            expected: engines
                .help_out(const_decl_signature.type_ascription.type_id)
                .to_string(),
        });
        return err(warnings, errors);
    }

    if errors.is_empty() {
        ok(const_decl, warnings, errors)
    } else {
        err(warnings, errors)
    }
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
    engines: &Engines,
    type_parameters: &[TypeParameter],
    trait_type_arguments: &[TypeArgument],
    self_type: TypeId,
) -> CompileResult<()> {
    let warnings = vec![];
    let mut errors = vec![];

    // create a list of defined generics, with the generic and a span
    let mut defined_generics: HashMap<_, _> = HashMap::from_iter(
        type_parameters
            .iter()
            .map(|x| (engines.te().get(x.type_id), x.span()))
            .map(|(thing, sp)| (WithEngines::new(thing, engines), sp)),
    );

    // create a list of the generics in use in the impl signature
    let mut generics_in_use = HashSet::new();
    for type_arg in trait_type_arguments.iter() {
        generics_in_use.extend(
            engines
                .te()
                .get(type_arg.type_id)
                .extract_nested_generics(engines),
        );
    }
    generics_in_use.extend(engines.te().get(self_type).extract_nested_generics(engines));

    // TODO: add a lookup in the trait constraints here and add it to
    // generics_in_use

    // deduct the generics in use from the defined generics
    for generic in generics_in_use.into_iter() {
        defined_generics.remove(&generic);
    }

    // create an error for all of the leftover generics
    for (k, v) in defined_generics.into_iter() {
        errors.push(CompileError::UnconstrainedGenericParameter {
            ty: format!("{k}"),
            span: v,
        });
    }

    if errors.is_empty() {
        ok((), warnings, errors)
    } else {
        err(warnings, errors)
    }
}

fn handle_supertraits(
    mut ctx: TypeCheckContext,
    supertraits: &[Supertrait],
) -> CompileResult<(InterfaceItemMap, ItemMap)> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    let decl_engine = ctx.engines.de();

    let mut interface_surface_item_ids: InterfaceItemMap = BTreeMap::new();
    let mut impld_item_refs: ItemMap = BTreeMap::new();
    let self_type = ctx.self_type();

    for supertrait in supertraits.iter() {
        // Right now we don't have the ability to support defining a supertrait
        // using a callpath directly, so we check to see if the user has done
        // this and we disallow it.
        if !supertrait.name.prefixes.is_empty() {
            errors.push(CompileError::UnimplementedWithHelp(
                "Using module paths to define supertraits is not supported yet.",
                "try importing the trait with a \"use\" statement instead",
                supertrait.span(),
            ));
            continue;
        }

        match ctx
            .namespace
            .resolve_call_path(&supertrait.name)
            .ok(&mut warnings, &mut errors)
            .cloned()
        {
            Some(ty::TyDecl::TraitDecl(ty::TraitDecl { decl_id, .. })) => {
                let trait_decl = decl_engine.get_trait(&decl_id);

                // Right now we don't parse type arguments for supertraits, so
                // we should give this error message to users.
                if !trait_decl.type_parameters.is_empty() {
                    errors.push(CompileError::Unimplemented(
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
                let (next_interface_supertrait_decl_refs, next_these_supertrait_decl_refs) = check!(
                    handle_supertraits(ctx.by_ref(), &trait_decl.supertraits),
                    continue,
                    warnings,
                    errors
                );
                interface_surface_item_ids.extend(next_interface_supertrait_decl_refs);
                impld_item_refs.extend(next_these_supertrait_decl_refs);
            }
            Some(ty::TyDecl::AbiDecl { .. }) => errors.push(CompileError::AbiAsSupertrait {
                span: supertrait.name.span().clone(),
            }),
            _ => errors.push(CompileError::TraitNotFound {
                name: supertrait.name.to_string(),
                span: supertrait.name.span(),
            }),
        }
    }

    if errors.is_empty() {
        ok(
            (interface_surface_item_ids, impld_item_refs),
            warnings,
            errors,
        )
    } else {
        err(warnings, errors)
    }
}
