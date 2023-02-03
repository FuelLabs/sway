use std::collections::{BTreeMap, HashMap, HashSet};

use sway_error::error::{CompileError, InterfaceName};
use sway_types::{Ident, Span, Spanned};

use crate::{
    decl_engine::*,
    engine_threading::*,
    error::*,
    language::{parsed::*, ty, *},
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
            type_implementing_for,
            type_implementing_for_span,
            functions,
            block_span,
        } = impl_trait;

        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;
        let engines = ctx.engines();

        // create a namespace for the impl
        let mut impl_namespace = ctx.namespace.clone();
        let mut ctx = ctx.by_ref().scoped(&mut impl_namespace).allow_functions();

        // type check the type parameters which also inserts them into the namespace
        let mut new_impl_type_parameters = vec![];
        for type_parameter in impl_type_parameters.into_iter() {
            if !type_parameter.trait_constraints.is_empty() {
                errors.push(CompileError::WhereClauseNotYetSupported {
                    span: type_parameter.trait_constraints_span,
                });
                return err(warnings, errors);
            }
            new_impl_type_parameters.push(check!(
                TypeParameter::type_check(ctx.by_ref(), type_parameter),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

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
        let implementing_for_type_id = check!(
            ctx.resolve_type_without_self(
                type_engine.insert(decl_engine, type_implementing_for),
                &type_implementing_for_span,
                None
            ),
            return err(warnings, errors),
            warnings,
            errors
        );

        // check to see if this type is supported in impl blocks
        check!(
            type_engine
                .get(implementing_for_type_id)
                .expect_is_supported_in_impl_blocks_self(&type_implementing_for_span),
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
                implementing_for_type_id,
                &type_implementing_for_span
            ),
            return err(warnings, errors),
            warnings,
            errors
        );

        // Update the context with the new `self` type.
        let mut ctx = ctx
            .with_self_type(implementing_for_type_id)
            .with_help_text("")
            .with_type_annotation(type_engine.insert(decl_engine, TypeInfo::Unknown));

        let impl_trait = match ctx
            .namespace
            .resolve_call_path(&trait_name)
            .ok(&mut warnings, &mut errors)
            .cloned()
        {
            Some(ty::TyDeclaration::TraitDeclaration(decl_id)) => {
                let mut trait_decl = check!(
                    CompileResult::from(decl_engine.get_trait(decl_id.clone(), &trait_name.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

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

                let new_methods = check!(
                    type_check_trait_implementation(
                        ctx.by_ref(),
                        &new_impl_type_parameters,
                        &trait_decl.type_parameters,
                        &trait_type_arguments,
                        &trait_decl.supertraits,
                        &trait_decl.interface_surface,
                        &trait_decl.methods,
                        &functions,
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
                    trait_decl_id: Some(decl_id),
                    span: block_span,
                    methods: new_methods,
                    implementing_for_type_id,
                    type_implementing_for_span: type_implementing_for_span.clone(),
                }
            }
            Some(ty::TyDeclaration::AbiDeclaration(decl_id)) => {
                // if you are comparing this with the `impl_trait` branch above, note that
                // there are no type arguments here because we don't support generic types
                // in contract ABIs yet (or ever?) due to the complexity of communicating
                // the ABI layout in the descriptor file.

                let abi = check!(
                    CompileResult::from(decl_engine.get_abi(decl_id.clone(), &trait_name.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                if !type_engine
                    .get(implementing_for_type_id)
                    .eq(&TypeInfo::Contract, engines)
                {
                    errors.push(CompileError::ImplAbiForNonContract {
                        span: type_implementing_for_span.clone(),
                        ty: engines.help_out(implementing_for_type_id).to_string(),
                    });
                }

                let mut ctx = ctx.with_mode(Mode::ImplAbiFn);

                let new_methods = check!(
                    type_check_trait_implementation(
                        ctx.by_ref(),
                        &[], // this is empty because abi definitions don't support generics,
                        &[], // this is empty because abi definitions don't support generics,
                        &[], // this is empty because abi definitions don't support generics,
                        &[], // this is empty because abi definitions don't support supertraits,
                        &abi.interface_surface,
                        &abi.methods,
                        &functions,
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
                    trait_decl_id: Some(decl_id),
                    span: block_span,
                    methods: new_methods,
                    implementing_for_type_id,
                    type_implementing_for_span,
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

    // If any method contains a call to get_storage_index, then
    // impl_typ can only be a storage type.
    // This is noted down in the type engine.
    fn gather_storage_only_types(
        engines: Engines<'_>,
        impl_typ: TypeId,
        methods: &[ty::TyFunctionDeclaration],
        access_span: &Span,
    ) -> Result<(), CompileError> {
        fn ast_node_contains_get_storage_index(
            decl_engine: &DeclEngine,
            x: &ty::TyAstNodeContent,
            access_span: &Span,
        ) -> Result<bool, CompileError> {
            match x {
                ty::TyAstNodeContent::Expression(expr)
                | ty::TyAstNodeContent::ImplicitReturnExpression(expr) => {
                    expr_contains_get_storage_index(decl_engine, expr, access_span)
                }
                ty::TyAstNodeContent::Declaration(decl) => {
                    decl_contains_get_storage_index(decl_engine, decl, access_span)
                }
                ty::TyAstNodeContent::SideEffect => Ok(false),
            }
        }

        fn expr_contains_get_storage_index(
            decl_engine: &DeclEngine,
            expr: &ty::TyExpression,
            access_span: &Span,
        ) -> Result<bool, CompileError> {
            let res = match &expr.expression {
                ty::TyExpressionVariant::Literal(_)
                | ty::TyExpressionVariant::VariableExpression { .. }
                | ty::TyExpressionVariant::FunctionParameter
                | ty::TyExpressionVariant::AsmExpression { .. }
                | ty::TyExpressionVariant::Break
                | ty::TyExpressionVariant::Continue
                | ty::TyExpressionVariant::StorageAccess(_)
                | ty::TyExpressionVariant::AbiName(_) => false,
                ty::TyExpressionVariant::FunctionApplication { arguments, .. } => {
                    for f in arguments.iter() {
                        let b = expr_contains_get_storage_index(decl_engine, &f.1, access_span)?;
                        if b {
                            return Ok(true);
                        }
                    }
                    false
                }
                ty::TyExpressionVariant::LazyOperator {
                    lhs: expr1,
                    rhs: expr2,
                    ..
                }
                | ty::TyExpressionVariant::ArrayIndex {
                    prefix: expr1,
                    index: expr2,
                } => {
                    expr_contains_get_storage_index(decl_engine, expr1, access_span)?
                        || expr_contains_get_storage_index(decl_engine, expr2, access_span)?
                }
                ty::TyExpressionVariant::Tuple { fields: exprvec }
                | ty::TyExpressionVariant::Array { contents: exprvec } => {
                    for f in exprvec.iter() {
                        let b = expr_contains_get_storage_index(decl_engine, f, access_span)?;
                        if b {
                            return Ok(true);
                        }
                    }
                    false
                }

                ty::TyExpressionVariant::StructExpression { fields, .. } => {
                    for f in fields.iter() {
                        let b =
                            expr_contains_get_storage_index(decl_engine, &f.value, access_span)?;
                        if b {
                            return Ok(true);
                        }
                    }
                    false
                }
                ty::TyExpressionVariant::CodeBlock(cb) => {
                    codeblock_contains_get_storage_index(decl_engine, cb, access_span)?
                }
                ty::TyExpressionVariant::IfExp {
                    condition,
                    then,
                    r#else,
                } => {
                    expr_contains_get_storage_index(decl_engine, condition, access_span)?
                        || expr_contains_get_storage_index(decl_engine, then, access_span)?
                        || r#else.as_ref().map_or(Ok(false), |r#else| {
                            expr_contains_get_storage_index(decl_engine, r#else, access_span)
                        })?
                }
                ty::TyExpressionVariant::StructFieldAccess { prefix: exp, .. }
                | ty::TyExpressionVariant::TupleElemAccess { prefix: exp, .. }
                | ty::TyExpressionVariant::AbiCast { address: exp, .. }
                | ty::TyExpressionVariant::EnumTag { exp }
                | ty::TyExpressionVariant::UnsafeDowncast { exp, .. } => {
                    expr_contains_get_storage_index(decl_engine, exp, access_span)?
                }
                ty::TyExpressionVariant::EnumInstantiation { contents, .. } => {
                    contents.as_ref().map_or(Ok(false), |f| {
                        expr_contains_get_storage_index(decl_engine, f, access_span)
                    })?
                }

                ty::TyExpressionVariant::IntrinsicFunction(ty::TyIntrinsicFunctionKind {
                    kind,
                    ..
                }) => matches!(kind, sway_ast::intrinsics::Intrinsic::GetStorageKey),
                ty::TyExpressionVariant::WhileLoop { condition, body } => {
                    expr_contains_get_storage_index(decl_engine, condition, access_span)?
                        || codeblock_contains_get_storage_index(decl_engine, body, access_span)?
                }
                ty::TyExpressionVariant::Reassignment(reassignment) => {
                    expr_contains_get_storage_index(decl_engine, &reassignment.rhs, access_span)?
                }
                ty::TyExpressionVariant::StorageReassignment(storage_reassignment) => {
                    expr_contains_get_storage_index(
                        decl_engine,
                        &storage_reassignment.rhs,
                        access_span,
                    )?
                }
                ty::TyExpressionVariant::Return(exp) => {
                    expr_contains_get_storage_index(decl_engine, exp, access_span)?
                }
            };
            Ok(res)
        }

        fn decl_contains_get_storage_index(
            decl_engine: &DeclEngine,
            decl: &ty::TyDeclaration,
            access_span: &Span,
        ) -> Result<bool, CompileError> {
            match decl {
                ty::TyDeclaration::VariableDeclaration(decl) => {
                    expr_contains_get_storage_index(decl_engine, &decl.body, access_span)
                }
                ty::TyDeclaration::ConstantDeclaration(decl_id) => {
                    let ty::TyConstantDeclaration { value: expr, .. } =
                        decl_engine.get_constant(decl_id.clone(), access_span)?;
                    expr_contains_get_storage_index(decl_engine, &expr, access_span)
                }
                // We're already inside a type's impl. So we can't have these
                // nested functions etc. We just ignore them.
                ty::TyDeclaration::FunctionDeclaration(_)
                | ty::TyDeclaration::TraitDeclaration(_)
                | ty::TyDeclaration::StructDeclaration(_)
                | ty::TyDeclaration::EnumDeclaration(_)
                | ty::TyDeclaration::ImplTrait(_)
                | ty::TyDeclaration::AbiDeclaration(_)
                | ty::TyDeclaration::GenericTypeForFunctionScope { .. }
                | ty::TyDeclaration::ErrorRecovery(_)
                | ty::TyDeclaration::StorageDeclaration(_) => Ok(false),
            }
        }

        fn codeblock_contains_get_storage_index(
            decl_engine: &DeclEngine,
            cb: &ty::TyCodeBlock,
            access_span: &Span,
        ) -> Result<bool, CompileError> {
            for content in cb.contents.iter() {
                let b = ast_node_contains_get_storage_index(
                    decl_engine,
                    &content.content,
                    access_span,
                )?;
                if b {
                    return Ok(true);
                }
            }
            Ok(false)
        }

        let type_engine = engines.te();
        let decl_engine = engines.de();

        for method in methods.iter() {
            let contains_get_storage_index =
                codeblock_contains_get_storage_index(decl_engine, &method.body, access_span)?;
            if contains_get_storage_index {
                type_engine.set_type_as_storage_only(impl_typ);
                return Ok(());
            }
        }

        Ok(())
    }

    pub(crate) fn type_check_impl_self(
        ctx: TypeCheckContext,
        impl_self: ImplSelf,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let ImplSelf {
            impl_type_parameters,
            type_implementing_for,
            type_implementing_for_span,
            functions,
            block_span,
        } = impl_self;

        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;
        let engines = ctx.engines();

        // create the namespace for the impl
        let mut impl_namespace = ctx.namespace.clone();
        let mut ctx = ctx.scoped(&mut impl_namespace).allow_functions();

        // create the trait name
        let trait_name = CallPath {
            prefixes: vec![],
            suffix: match &type_implementing_for {
                TypeInfo::Custom { name, .. } => name.clone(),
                _ => Ident::new_with_override("r#Self", type_implementing_for_span.clone()),
            },
            is_absolute: false,
        };

        // type check the type parameters which also inserts them into the namespace
        let mut new_impl_type_parameters = vec![];
        for type_parameter in impl_type_parameters.into_iter() {
            if !type_parameter.trait_constraints.is_empty() {
                errors.push(CompileError::WhereClauseNotYetSupported {
                    span: type_parameter.trait_constraints_span,
                });
                continue;
            }
            new_impl_type_parameters.push(check!(
                TypeParameter::type_check(ctx.by_ref(), type_parameter),
                continue,
                warnings,
                errors
            ));
        }
        if !errors.is_empty() {
            return err(warnings, errors);
        }

        // type check the type that we are implementing for
        let implementing_for_type_id = check!(
            ctx.resolve_type_without_self(
                type_engine.insert(decl_engine, type_implementing_for),
                &type_implementing_for_span,
                None
            ),
            return err(warnings, errors),
            warnings,
            errors
        );

        // check to see if this type is supported in impl blocks
        check!(
            type_engine
                .get(implementing_for_type_id)
                .expect_is_supported_in_impl_blocks_self(&type_implementing_for_span),
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
                implementing_for_type_id,
                &type_implementing_for_span
            ),
            return err(warnings, errors),
            warnings,
            errors
        );

        let mut ctx = ctx
            .with_self_type(implementing_for_type_id)
            .with_help_text("")
            .with_type_annotation(type_engine.insert(decl_engine, TypeInfo::Unknown));

        // type check the methods inside of the impl block
        let mut methods = vec![];
        for fn_decl in functions.into_iter() {
            methods.push(check!(
                ty::TyFunctionDeclaration::type_check(ctx.by_ref(), fn_decl, true, true),
                continue,
                warnings,
                errors
            ));
        }
        if !errors.is_empty() {
            return err(warnings, errors);
        }

        check!(
            CompileResult::from(Self::gather_storage_only_types(
                engines,
                implementing_for_type_id,
                &methods,
                &type_implementing_for_span,
            )),
            return err(warnings, errors),
            warnings,
            errors
        );

        let methods_ids = methods
            .iter()
            .map(|d| decl_engine.insert(d.clone()))
            .collect::<Vec<_>>();

        let impl_trait = ty::TyImplTrait {
            impl_type_parameters: new_impl_type_parameters,
            trait_name,
            trait_type_arguments: vec![], // this is empty because impl selfs don't support generics on the "Self" trait,
            trait_decl_id: None,
            span: block_span,
            methods: methods_ids,
            implementing_for_type_id,
            type_implementing_for_span,
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
    trait_interface_surface: &[DeclId],
    trait_methods: &[DeclId],
    impl_methods: &[FunctionDeclaration],
    trait_name: &CallPath,
    block_span: &Span,
    is_contract: bool,
) -> CompileResult<Vec<DeclId>> {
    let mut errors = vec![];
    let mut warnings = vec![];

    let decl_engine = ctx.decl_engine;
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

    // Gather the supertrait "stub_method_ids" and "impld_method_ids".
    let (supertrait_stub_method_ids, supertrait_impld_method_ids) = check!(
        handle_supertraits(ctx.by_ref(), trait_supertraits),
        return err(warnings, errors),
        warnings,
        errors
    );

    // Insert the implemented methods for the supertraits into this namespace
    // so that the methods defined in the impl block can use them.
    //
    // We purposefully do not check for errors here because this is a temporary
    // namespace and not a real impl block defined by the user.
    ctx.namespace.insert_trait_implementation(
        trait_name.clone(),
        trait_type_arguments.to_vec(),
        self_type,
        &supertrait_impld_method_ids
            .values()
            .cloned()
            .collect::<Vec<_>>(),
        &trait_name.span(),
        false,
        engines,
    );

    // This map keeps track of the remaining functions in the interface surface
    // that still need to be implemented for the trait to be fully implemented.
    let mut method_checklist: BTreeMap<Ident, ty::TyTraitFn> = BTreeMap::new();

    // This map keeps track of the stub declaration id's of the trait
    // definition.
    let mut stub_method_ids: BTreeMap<Ident, DeclId> = BTreeMap::new();

    // This map keeps track of the new declaration ids of the implemented
    // interface surface.
    let mut impld_method_ids: BTreeMap<Ident, DeclId> = BTreeMap::new();

    for decl_id in trait_interface_surface.iter() {
        let method = check!(
            CompileResult::from(decl_engine.get_trait_fn(decl_id.clone(), block_span)),
            return err(warnings, errors),
            warnings,
            errors
        );
        let name = method.name.clone();

        // Add this method to the checklist.
        method_checklist.insert(name.clone(), method);

        // Add this method to the "stub methods".
        stub_method_ids.insert(name, decl_id.clone());
    }

    for impl_method in impl_methods {
        let impl_method = check!(
            type_check_impl_method(
                ctx.by_ref(),
                impl_type_parameters,
                impl_method,
                trait_name,
                is_contract,
                &impld_method_ids,
                &method_checklist
            ),
            ty::TyFunctionDeclaration::error(impl_method.clone(), engines),
            warnings,
            errors
        );
        let name = impl_method.name.clone();
        let decl_id = decl_engine.insert(impl_method);

        // Remove this method from the checklist.
        method_checklist.remove(&name);

        // Add this method to the "impld methods".
        impld_method_ids.insert(name, decl_id);
    }

    let mut all_method_ids: Vec<DeclId> = impld_method_ids.values().cloned().collect();

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
    stub_method_ids.extend(supertrait_stub_method_ids);
    impld_method_ids.extend(supertrait_impld_method_ids);
    let decl_mapping = DeclMapping::from_stub_and_impld_decl_ids(stub_method_ids, impld_method_ids);
    for decl_id in trait_methods.iter() {
        let mut method = check!(
            CompileResult::from(decl_engine.get_function(decl_id.clone(), block_span)),
            return err(warnings, errors),
            warnings,
            errors
        );
        method.replace_decls(&decl_mapping, engines);
        method.subst(&type_mapping, engines);
        method.replace_self_type(engines, ctx.self_type());
        all_method_ids.push(
            decl_engine
                .insert(method)
                .with_parent(decl_engine, decl_id.clone()),
        );
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

    if errors.is_empty() {
        ok(all_method_ids, warnings, errors)
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
    impld_method_ids: &BTreeMap<Ident, DeclId>,
    method_checklist: &BTreeMap<Ident, ty::TyTraitFn>,
) -> CompileResult<ty::TyFunctionDeclaration> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let type_engine = ctx.type_engine;
    let decl_engine = ctx.decl_engine;
    let engines = ctx.engines();
    let self_type = ctx.self_type();

    let mut ctx = ctx
        .by_ref()
        .with_help_text("")
        .with_type_annotation(type_engine.insert(decl_engine, TypeInfo::Unknown));

    let interface_name = || -> InterfaceName {
        if is_contract {
            InterfaceName::Abi(trait_name.suffix.clone())
        } else {
            InterfaceName::Trait(trait_name.suffix.clone())
        }
    };

    // type check the function declaration
    let mut impl_method = check!(
        ty::TyFunctionDeclaration::type_check(ctx.by_ref(), impl_method.clone(), true, false),
        return err(warnings, errors),
        warnings,
        errors
    );

    // Ensure that there aren't multiple definitions of this function impl'd
    if impld_method_ids.contains_key(&impl_method.name.clone()) {
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

        if !type_engine.get(impl_method_param.type_id).eq(
            &type_engine.get(impl_method_signature_param.type_id),
            engines,
        ) {
            errors.push(CompileError::MismatchedTypeInInterfaceSurface {
                interface_name: interface_name(),
                span: impl_method_param.type_span.clone(),
                given: engines.help_out(impl_method_param.type_id).to_string(),
                expected: engines
                    .help_out(impl_method_signature_param.type_id)
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

    if !type_engine
        .get(impl_method.return_type)
        .eq(&type_engine.get(impl_method_signature.return_type), engines)
    {
        errors.push(CompileError::MismatchedTypeInInterfaceSurface {
            interface_name: interface_name(),
            span: impl_method.return_type_span.clone(),
            expected: engines
                .help_out(impl_method_signature.return_type)
                .to_string(),
            given: engines.help_out(impl_method.return_type).to_string(),
        });
        return err(warnings, errors);
    }

    // if this method uses a type parameter from its parent's impl type
    // parameters that is not constrained by the type that we are
    // implementing for, then we need to add that type parameter to the
    // method's type parameters so that in-line monomorphization can
    // complete.
    //
    // NOTE: this is a semi-hack that is used to force monomorphization of
    // trait methods that contain a generic defined in the parent impl...
    // without stuffing the generic into the method's type parameters, its
    // not currently possible to monomorphize on that generic at function
    // application time.
    //
    // *This will change* when either https://github.com/FuelLabs/sway/issues/1267
    // or https://github.com/FuelLabs/sway/issues/2814 goes in.
    let unconstrained_type_parameters_in_this_function: HashSet<WithEngines<'_, TypeParameter>> =
        impl_method
            .unconstrained_type_parameters(engines, impl_type_parameters)
            .into_iter()
            .cloned()
            .map(|x| WithEngines::new(x, engines))
            .collect();
    let unconstrained_type_parameters_in_the_type: HashSet<WithEngines<'_, TypeParameter>> =
        self_type
            .unconstrained_type_parameters(engines, impl_type_parameters)
            .into_iter()
            .cloned()
            .map(|x| WithEngines::new(x, engines))
            .collect::<HashSet<_>>();
    let mut unconstrained_type_parameters_to_be_added =
        unconstrained_type_parameters_in_this_function
            .difference(&unconstrained_type_parameters_in_the_type)
            .cloned()
            .into_iter()
            .map(|x| x.thing)
            .collect::<Vec<_>>();
    impl_method
        .type_parameters
        .append(&mut unconstrained_type_parameters_to_be_added);

    if errors.is_empty() {
        ok(impl_method, warnings, errors)
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
    engines: Engines<'_>,
    type_parameters: &[TypeParameter],
    trait_type_arguments: &[TypeArgument],
    self_type: TypeId,
    self_type_span: &Span,
) -> CompileResult<()> {
    let mut warnings = vec![];
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
        generics_in_use.extend(check!(
            engines
                .te()
                .get(type_arg.type_id)
                .extract_nested_generics(engines, &type_arg.span),
            HashSet::new(),
            warnings,
            errors
        ));
    }
    generics_in_use.extend(check!(
        engines
            .te()
            .get(self_type)
            .extract_nested_generics(engines, self_type_span),
        HashSet::new(),
        warnings,
        errors
    ));

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
) -> CompileResult<(BTreeMap<Ident, DeclId>, BTreeMap<Ident, DeclId>)> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    let decl_engine = ctx.decl_engine;

    let mut interface_surface_methods_ids: BTreeMap<Ident, DeclId> = BTreeMap::new();
    let mut impld_method_ids: BTreeMap<Ident, DeclId> = BTreeMap::new();
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
            Some(ty::TyDeclaration::TraitDeclaration(decl_id)) => {
                let trait_decl = check!(
                    CompileResult::from(decl_engine.get_trait(decl_id.clone(), &supertrait.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

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
                let (trait_interface_surface_methods_ids, trait_impld_method_ids) = check!(
                    trait_decl.retrieve_interface_surface_and_implemented_methods_for_type(
                        ctx.by_ref(),
                        self_type,
                        &supertrait.name
                    ),
                    continue,
                    warnings,
                    errors
                );
                interface_surface_methods_ids.extend(trait_interface_surface_methods_ids);
                impld_method_ids.extend(trait_impld_method_ids);

                // Retrieve the interface surfaces and implemented methods for
                // the supertraits of this type.
                let (next_stub_supertrait_decl_ids, next_these_supertrait_decl_ids) = check!(
                    handle_supertraits(ctx.by_ref(), &trait_decl.supertraits),
                    continue,
                    warnings,
                    errors
                );
                interface_surface_methods_ids.extend(next_stub_supertrait_decl_ids);
                impld_method_ids.extend(next_these_supertrait_decl_ids);
            }
            Some(ty::TyDeclaration::AbiDeclaration(_)) => {
                errors.push(CompileError::AbiAsSupertrait {
                    span: supertrait.name.span().clone(),
                })
            }
            _ => errors.push(CompileError::TraitNotFound {
                name: supertrait.name.to_string(),
                span: supertrait.name.span(),
            }),
        }
    }

    if errors.is_empty() {
        ok(
            (interface_surface_methods_ids, impld_method_ids),
            warnings,
            errors,
        )
    } else {
        err(warnings, errors)
    }
}
