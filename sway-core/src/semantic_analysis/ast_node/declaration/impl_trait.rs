use std::collections::{BTreeMap, HashMap, HashSet};

use sway_error::error::CompileError;
use sway_types::{Ident, Span, Spanned};

use crate::{
    declaration_engine::{declaration_engine::*, DeclMapping, DeclarationId, ReplaceDecls},
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

        // create a namespace for the impl
        let mut impl_namespace = ctx.namespace.clone();
        let mut ctx = ctx.by_ref().scoped(&mut impl_namespace);

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
                type_engine.insert_type(type_implementing_for),
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
                .look_up_type_id(implementing_for_type_id)
                .expect_is_supported_in_impl_blocks_self(&type_implementing_for_span),
            return err(warnings, errors),
            warnings,
            errors
        );

        // check for unconstrained type parameters
        check!(
            check_for_unconstrained_type_parameters(
                type_engine,
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
            .with_type_annotation(type_engine.insert_type(TypeInfo::Unknown));

        let impl_trait = match ctx
            .namespace
            .resolve_call_path(&trait_name)
            .ok(&mut warnings, &mut errors)
            .cloned()
        {
            Some(ty::TyDeclaration::TraitDeclaration(decl_id)) => {
                let mut trait_decl = check!(
                    CompileResult::from(de_get_trait(decl_id, &trait_name.span())),
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
                    CompileResult::from(de_get_abi(decl_id, &trait_name.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                if type_engine.look_up_type_id(implementing_for_type_id) != TypeInfo::Contract {
                    errors.push(CompileError::ImplAbiForNonContract {
                        span: type_implementing_for_span.clone(),
                        ty: implementing_for_type_id.to_string(),
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
        type_engine: &TypeEngine,
        impl_typ: TypeId,
        methods: &[ty::TyFunctionDeclaration],
        access_span: &Span,
    ) -> Result<(), CompileError> {
        fn ast_node_contains_get_storage_index(
            x: &ty::TyAstNodeContent,
            access_span: &Span,
        ) -> Result<bool, CompileError> {
            match x {
                ty::TyAstNodeContent::Expression(expr)
                | ty::TyAstNodeContent::ImplicitReturnExpression(expr) => {
                    expr_contains_get_storage_index(expr, access_span)
                }
                ty::TyAstNodeContent::Declaration(decl) => {
                    decl_contains_get_storage_index(decl, access_span)
                }
                ty::TyAstNodeContent::SideEffect => Ok(false),
            }
        }

        fn expr_contains_get_storage_index(
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
                        let b = expr_contains_get_storage_index(&f.1, access_span)?;
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
                    expr_contains_get_storage_index(expr1, access_span)?
                        || expr_contains_get_storage_index(expr2, access_span)?
                }
                ty::TyExpressionVariant::Tuple { fields: exprvec }
                | ty::TyExpressionVariant::Array { contents: exprvec } => {
                    for f in exprvec.iter() {
                        let b = expr_contains_get_storage_index(f, access_span)?;
                        if b {
                            return Ok(true);
                        }
                    }
                    false
                }

                ty::TyExpressionVariant::StructExpression { fields, .. } => {
                    for f in fields.iter() {
                        let b = expr_contains_get_storage_index(&f.value, access_span)?;
                        if b {
                            return Ok(true);
                        }
                    }
                    false
                }
                ty::TyExpressionVariant::CodeBlock(cb) => {
                    codeblock_contains_get_storage_index(cb, access_span)?
                }
                ty::TyExpressionVariant::IfExp {
                    condition,
                    then,
                    r#else,
                } => {
                    expr_contains_get_storage_index(condition, access_span)?
                        || expr_contains_get_storage_index(then, access_span)?
                        || r#else.as_ref().map_or(Ok(false), |r#else| {
                            expr_contains_get_storage_index(r#else, access_span)
                        })?
                }
                ty::TyExpressionVariant::StructFieldAccess { prefix: exp, .. }
                | ty::TyExpressionVariant::TupleElemAccess { prefix: exp, .. }
                | ty::TyExpressionVariant::AbiCast { address: exp, .. }
                | ty::TyExpressionVariant::EnumTag { exp }
                | ty::TyExpressionVariant::UnsafeDowncast { exp, .. } => {
                    expr_contains_get_storage_index(exp, access_span)?
                }
                ty::TyExpressionVariant::EnumInstantiation { contents, .. } => {
                    contents.as_ref().map_or(Ok(false), |f| {
                        expr_contains_get_storage_index(f, access_span)
                    })?
                }

                ty::TyExpressionVariant::IntrinsicFunction(ty::TyIntrinsicFunctionKind {
                    kind,
                    ..
                }) => matches!(kind, sway_ast::intrinsics::Intrinsic::GetStorageKey),
                ty::TyExpressionVariant::WhileLoop { condition, body } => {
                    expr_contains_get_storage_index(condition, access_span)?
                        || codeblock_contains_get_storage_index(body, access_span)?
                }
                ty::TyExpressionVariant::Reassignment(reassignment) => {
                    expr_contains_get_storage_index(&reassignment.rhs, access_span)?
                }
                ty::TyExpressionVariant::StorageReassignment(storage_reassignment) => {
                    expr_contains_get_storage_index(&storage_reassignment.rhs, access_span)?
                }
                ty::TyExpressionVariant::Return(exp) => {
                    expr_contains_get_storage_index(exp, access_span)?
                }
            };
            Ok(res)
        }

        fn decl_contains_get_storage_index(
            decl: &ty::TyDeclaration,
            access_span: &Span,
        ) -> Result<bool, CompileError> {
            match decl {
                ty::TyDeclaration::VariableDeclaration(decl) => {
                    expr_contains_get_storage_index(&decl.body, access_span)
                }
                ty::TyDeclaration::ConstantDeclaration(decl_id) => {
                    let ty::TyConstantDeclaration { value: expr, .. } =
                        de_get_constant(decl_id.clone(), access_span)?;
                    expr_contains_get_storage_index(&expr, access_span)
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
            cb: &ty::TyCodeBlock,
            access_span: &Span,
        ) -> Result<bool, CompileError> {
            for content in cb.contents.iter() {
                let b = ast_node_contains_get_storage_index(&content.content, access_span)?;
                if b {
                    return Ok(true);
                }
            }
            Ok(false)
        }

        for method in methods.iter() {
            let contains_get_storage_index =
                codeblock_contains_get_storage_index(&method.body, access_span)?;
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

        // create the namespace for the impl
        let mut impl_namespace = ctx.namespace.clone();
        let mut ctx = ctx.scoped(&mut impl_namespace);

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
                type_engine.insert_type(type_implementing_for),
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
                .look_up_type_id(implementing_for_type_id)
                .expect_is_supported_in_impl_blocks_self(&type_implementing_for_span),
            return err(warnings, errors),
            warnings,
            errors
        );

        // check for unconstrained type parameters
        check!(
            check_for_unconstrained_type_parameters(
                type_engine,
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
            .with_type_annotation(type_engine.insert_type(TypeInfo::Unknown));

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
                type_engine,
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
            .map(|d| de_insert_function(d.clone()))
            .collect::<Vec<_>>();

        let impl_trait = ty::TyImplTrait {
            impl_type_parameters: new_impl_type_parameters,
            trait_name,
            trait_type_arguments: vec![], // this is empty because impl selfs don't support generics on the "Self" trait,
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
    trait_interface_surface: &[DeclarationId],
    trait_methods: &[DeclarationId],
    impl_methods: &[FunctionDeclaration],
    trait_name: &CallPath,
    block_span: &Span,
    is_contract: bool,
) -> CompileResult<Vec<DeclarationId>> {
    use sway_error::error::InterfaceName;

    let mut errors = vec![];
    let mut warnings = vec![];

    let type_engine = ctx.type_engine;
    let self_type = ctx.self_type();
    let interface_name = || -> InterfaceName {
        if is_contract {
            InterfaceName::Abi(trait_name.suffix.clone())
        } else {
            InterfaceName::Trait(trait_name.suffix.clone())
        }
    };

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
                type_engine,
            ),
        return err(warnings, errors),
        warnings,
        errors
    );

    // Gather the supertrait "original_method_ids" and "impld_method_ids".
    let (supertrait_original_method_ids, supertrait_impld_method_ids) = check!(
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
        type_engine,
    );

    // This map keeps track of the remaining functions in the interface surface
    // that still need to be implemented for the trait to be fully implemented.
    let mut method_checklist: BTreeMap<Ident, ty::TyTraitFn> = BTreeMap::new();

    // This map keeps track of the original declaration id's of the original
    // interface surface.
    let mut original_method_ids: BTreeMap<Ident, DeclarationId> = BTreeMap::new();

    // This map keeps track of the new declaration ids of the implemented
    // interface surface.
    let mut impld_method_ids: BTreeMap<Ident, DeclarationId> = BTreeMap::new();

    for decl_id in trait_interface_surface.iter() {
        let method = check!(
            CompileResult::from(de_get_trait_fn(decl_id.clone(), block_span)),
            return err(warnings, errors),
            warnings,
            errors
        );
        let name = method.name.clone();
        method_checklist.insert(name.clone(), method);
        original_method_ids.insert(name, decl_id.clone());
    }

    for impl_method in impl_methods {
        let mut ctx = ctx
            .by_ref()
            .with_help_text("")
            .with_type_annotation(type_engine.insert_type(TypeInfo::Unknown));

        // type check the function declaration
        let mut impl_method = check!(
            ty::TyFunctionDeclaration::type_check(ctx.by_ref(), impl_method.clone(), true, false),
            continue,
            warnings,
            errors
        );

        // Ensure that there aren't multiple definitions of this function impl'd
        if impld_method_ids.contains_key(&impl_method.name.clone()) {
            errors.push(CompileError::MultipleDefinitionsOfFunction {
                name: impl_method.name.clone(),
            });
            continue;
        }

        // remove this function from the "checklist"
        let mut impl_method_signature = match method_checklist.remove(&impl_method.name) {
            Some(trait_fn) => trait_fn,
            None => {
                errors.push(CompileError::FunctionNotAPartOfInterfaceSurface {
                    name: impl_method.name.clone(),
                    interface_name: interface_name(),
                    span: impl_method.name.span(),
                });
                continue;
            }
        };

        // replace instances of `TypeInfo::SelfType` with a fresh
        // `TypeInfo::SelfType` to avoid replacing types in the original trait
        // declaration
        impl_method_signature.replace_self_type(type_engine, ctx.self_type());

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
            continue;
        }

        // unify the types from the parameters of the function declaration
        // with the parameters of the function signature
        for (impl_method_signature_param, impl_method_param) in impl_method_signature
            .parameters
            .iter()
            .zip(&impl_method.parameters)
        {
            // TODO use trait constraints as part of the type here to
            // implement trait constraint solver */
            // check if the mutability of the parameters is incompatible
            if impl_method_param.is_mutable != impl_method_signature_param.is_mutable {
                errors.push(CompileError::ParameterMutabilityMismatch {
                    span: impl_method_param.mutability_span.clone(),
                });
            }

            if (impl_method_param.is_reference || impl_method_signature_param.is_reference)
                && is_contract
            {
                errors.push(CompileError::RefMutParameterInContract {
                    span: impl_method_param.mutability_span.clone(),
                });
            }

            let (new_warnings, new_errors) = type_engine.unify_right_with_self(
                impl_method_param.type_id,
                impl_method_signature_param.type_id,
                ctx.self_type(),
                &impl_method_signature_param.type_span,
                ctx.help_text(),
            );
            if !new_warnings.is_empty() || !new_errors.is_empty() {
                errors.push(CompileError::MismatchedTypeInInterfaceSurface {
                    interface_name: interface_name(),
                    span: impl_method_param.type_span.clone(),
                    given: impl_method_param.type_id.to_string(),
                    expected: impl_method_signature_param.type_id.to_string(),
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

        // unify the return type of the implemented function and the return
        // type of the signature
        let (new_warnings, new_errors) = type_engine.unify_right_with_self(
            impl_method.return_type,
            impl_method_signature.return_type,
            ctx.self_type(),
            &impl_method.return_type_span,
            ctx.help_text(),
        );
        if !new_warnings.is_empty() || !new_errors.is_empty() {
            errors.push(CompileError::MismatchedTypeInInterfaceSurface {
                interface_name: interface_name(),
                span: impl_method.return_type_span.clone(),
                expected: impl_method_signature.return_type.to_string(),
                given: impl_method.return_type.to_string(),
            });
            continue;
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
        let unconstrained_type_parameters_in_this_function: HashSet<TypeParameter> = impl_method
            .unconstrained_type_parameters(type_engine, impl_type_parameters)
            .into_iter()
            .cloned()
            .collect();
        let unconstrained_type_parameters_in_the_type: HashSet<TypeParameter> = ctx
            .self_type()
            .unconstrained_type_parameters(type_engine, impl_type_parameters)
            .into_iter()
            .cloned()
            .collect::<HashSet<_>>();
        let mut unconstrained_type_parameters_to_be_added =
            unconstrained_type_parameters_in_this_function
                .difference(&unconstrained_type_parameters_in_the_type)
                .cloned()
                .into_iter()
                .collect::<Vec<_>>();
        impl_method
            .type_parameters
            .append(&mut unconstrained_type_parameters_to_be_added);

        let name = impl_method.name.clone();
        let decl_id = de_insert_function(impl_method);
        impld_method_ids.insert(name, decl_id);
    }

    let mut all_method_ids: Vec<DeclarationId> = impld_method_ids.values().cloned().collect();

    // Retrieve the methods defined on the trait declaration and transform
    // them into the correct typing for this impl block by using the type
    // parameters from the original trait declaration and the type arguments of
    // the trait name in the current impl block that we are type checking and
    // using the original decl ids from the interface surface and the new
    // decl ids from the newly implemented methods.
    let type_mapping = TypeMapping::from_type_parameters_and_type_arguments(
        trait_type_parameters
            .iter()
            .map(|type_param| type_param.type_id)
            .collect(),
        trait_type_arguments
            .iter()
            .map(|type_arg| type_arg.type_id)
            .collect(),
    );
    original_method_ids.extend(supertrait_original_method_ids);
    impld_method_ids.extend(supertrait_impld_method_ids);
    let decl_mapping =
        DeclMapping::from_original_and_new_decl_ids(original_method_ids, impld_method_ids);
    for decl_id in trait_methods.iter() {
        let mut method = check!(
            CompileResult::from(de_get_function(decl_id.clone(), block_span)),
            return err(warnings, errors),
            warnings,
            errors
        );
        method.replace_decls(&decl_mapping);
        method.copy_types(&type_mapping, ctx.type_engine);
        method.replace_self_type(type_engine, ctx.self_type());
        all_method_ids.push(de_insert_function(method).with_parent(decl_id.clone()));
    }

    // check that the implementation checklist is complete
    if !method_checklist.is_empty() {
        errors.push(CompileError::MissingInterfaceSurfaceMethods {
            span: block_span.clone(),
            missing_functions: method_checklist
                .into_iter()
                .map(|(ident, _)| ident.as_str().to_string())
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
    type_engine: &TypeEngine,
    type_parameters: &[TypeParameter],
    trait_type_arguments: &[TypeArgument],
    self_type: TypeId,
    self_type_span: &Span,
) -> CompileResult<()> {
    let mut warnings = vec![];
    let mut errors = vec![];

    // create a list of defined generics, with the generic and a span
    let mut defined_generics: HashMap<TypeInfo, Span> = HashMap::from_iter(
        type_parameters
            .iter()
            .map(|x| (type_engine.look_up_type_id(x.type_id), x.span())),
    );

    // create a list of the generics in use in the impl signature
    let mut generics_in_use = HashSet::new();
    for type_arg in trait_type_arguments.iter() {
        generics_in_use.extend(check!(
            type_engine
                .look_up_type_id(type_arg.type_id)
                .extract_nested_generics(type_engine, &type_arg.span),
            HashSet::new(),
            warnings,
            errors
        ));
    }
    generics_in_use.extend(check!(
        type_engine
            .look_up_type_id(self_type)
            .extract_nested_generics(type_engine, self_type_span),
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
            ty: format!("{}", type_engine.help_out(k)),
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
) -> CompileResult<(
    BTreeMap<Ident, DeclarationId>,
    BTreeMap<Ident, DeclarationId>,
)> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    let mut interface_surface_methods_ids: BTreeMap<Ident, DeclarationId> = BTreeMap::new();
    let mut impld_method_ids: BTreeMap<Ident, DeclarationId> = BTreeMap::new();
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
                    CompileResult::from(de_get_trait(decl_id.clone(), &supertrait.span())),
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
                let (next_original_supertrait_decl_ids, next_these_supertrait_decl_ids) = check!(
                    handle_supertraits(ctx.by_ref(), &trait_decl.supertraits),
                    continue,
                    warnings,
                    errors
                );
                interface_surface_methods_ids.extend(next_original_supertrait_decl_ids);
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
