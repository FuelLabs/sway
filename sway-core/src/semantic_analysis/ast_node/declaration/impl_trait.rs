use std::collections::{HashMap, HashSet};

use sway_error::error::CompileError;
use sway_types::{Ident, Span, Spanned};

use crate::{
    declaration_engine::{declaration_engine::*, DeclarationId},
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
                insert_type(type_implementing_for),
                &type_implementing_for_span,
                None
            ),
            return err(warnings, errors),
            warnings,
            errors
        );

        // check to see if this type is supported in impl blocks
        check!(
            look_up_type_id(implementing_for_type_id)
                .expect_is_supported_in_impl_blocks_self(&type_implementing_for_span),
            return err(warnings, errors),
            warnings,
            errors
        );

        // check for unconstrained type parameters
        check!(
            check_for_unconstrained_type_parameters(
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
            .with_type_annotation(insert_type(TypeInfo::Unknown));

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

                let functions_buf = check!(
                    type_check_trait_implementation(
                        ctx,
                        &new_impl_type_parameters,
                        &trait_type_arguments,
                        implementing_for_type_id,
                        &type_implementing_for_span,
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
                let functions_decl_id = functions_buf
                    .iter()
                    .map(|d| de_insert_function(d.clone()))
                    .collect::<Vec<_>>();
                ty::TyImplTrait {
                    impl_type_parameters: new_impl_type_parameters,
                    trait_name: trait_name.clone(),
                    trait_type_arguments,
                    span: block_span,
                    methods: functions_decl_id,
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

                if look_up_type_id(implementing_for_type_id) != TypeInfo::Contract {
                    errors.push(CompileError::ImplAbiForNonContract {
                        span: type_implementing_for_span.clone(),
                        ty: implementing_for_type_id.to_string(),
                    });
                }

                let ctx = ctx.with_mode(Mode::ImplAbiFn);

                let functions_buf = check!(
                    type_check_trait_implementation(
                        ctx,
                        &[], // this is empty because abi definitions don't support generics,
                        &[], // this is empty because abi definitions don't support generics,
                        implementing_for_type_id,
                        &type_implementing_for_span,
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
                let functions_decl_id = functions_buf
                    .iter()
                    .map(|d| de_insert_function(d.clone()))
                    .collect::<Vec<_>>();
                ty::TyImplTrait {
                    impl_type_parameters: vec![], // this is empty because abi definitions don't support generics
                    trait_name,
                    trait_type_arguments: vec![], // this is empty because abi definitions don't support generics
                    span: block_span,
                    methods: functions_decl_id,
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
                | ty::TyDeclaration::ErrorRecovery
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
                set_type_as_storage_only(impl_typ);
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
                return err(warnings, errors);
            }
            new_impl_type_parameters.push(check!(
                TypeParameter::type_check(ctx.by_ref(), type_parameter),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

        // type check the type that we are implementing for
        let implementing_for_type_id = check!(
            ctx.resolve_type_without_self(
                insert_type(type_implementing_for),
                &type_implementing_for_span,
                None
            ),
            return err(warnings, errors),
            warnings,
            errors
        );

        // check to see if this type is supported in impl blocks
        check!(
            look_up_type_id(implementing_for_type_id)
                .expect_is_supported_in_impl_blocks_self(&type_implementing_for_span),
            return err(warnings, errors),
            warnings,
            errors
        );

        // check for unconstrained type parameters
        check!(
            check_for_unconstrained_type_parameters(
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
            .with_type_annotation(insert_type(TypeInfo::Unknown));

        // type check the methods inside of the impl block
        let mut methods = vec![];
        for fn_decl in functions.into_iter() {
            methods.push(check!(
                ty::TyFunctionDeclaration::type_check(ctx.by_ref(), fn_decl, true),
                continue,
                warnings,
                errors
            ));
        }

        check!(
            CompileResult::from(Self::gather_storage_only_types(
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
    trait_type_arguments: &[TypeArgument],
    type_implementing_for: TypeId,
    type_implementing_for_span: &Span,
    trait_interface_surface: &[DeclarationId],
    trait_methods: &[FunctionDeclaration],
    functions: &[FunctionDeclaration],
    trait_name: &CallPath,
    block_span: &Span,
    is_contract: bool,
) -> CompileResult<Vec<ty::TyFunctionDeclaration>> {
    use sway_error::error::InterfaceName;
    let interface_name = || -> InterfaceName {
        if is_contract {
            InterfaceName::Abi(trait_name.suffix.clone())
        } else {
            InterfaceName::Trait(trait_name.suffix.clone())
        }
    };

    let mut errors = vec![];
    let mut warnings = vec![];

    let mut functions_buf: Vec<ty::TyFunctionDeclaration> = vec![];
    let mut processed_fns = std::collections::HashSet::<Ident>::new();

    let mut trait_fns = vec![];
    for decl_id in trait_interface_surface {
        match de_get_trait_fn(decl_id.clone(), block_span) {
            Ok(decl) => trait_fns.push(decl),
            Err(err) => errors.push(err),
        }
    }

    // this map keeps track of the remaining functions in the
    // interface surface that still need to be implemented for the
    // trait to be fully implemented
    let mut function_checklist: std::collections::BTreeMap<Ident, _> = trait_fns
        .into_iter()
        .map(|decl| (decl.name.clone(), decl))
        .collect();
    for fn_decl in functions {
        let mut ctx = ctx
            .by_ref()
            .with_help_text("")
            .with_type_annotation(insert_type(TypeInfo::Unknown));

        // type check the function declaration
        let mut fn_decl = check!(
            ty::TyFunctionDeclaration::type_check(ctx.by_ref(), fn_decl.clone(), true),
            continue,
            warnings,
            errors
        );

        // Ensure that there aren't multiple definitions of this function impl'd
        if !processed_fns.insert(fn_decl.name.clone()) {
            errors.push(CompileError::MultipleDefinitionsOfFunction {
                name: fn_decl.name.clone(),
            });
            return err(warnings, errors);
        }

        // remove this function from the "checklist"
        let mut fn_signature = match function_checklist.remove(&fn_decl.name) {
            Some(trait_fn) => trait_fn,
            None => {
                errors.push(CompileError::FunctionNotAPartOfInterfaceSurface {
                    name: fn_decl.name.clone(),
                    interface_name: interface_name(),
                    span: fn_decl.name.span(),
                });
                return err(warnings, errors);
            }
        };

        // replace instances of `TypeInfo::SelfType` with a fresh
        // `TypeInfo::SelfType` to avoid replacing types in the original trait
        // declaration
        fn_signature.replace_self_type(ctx.self_type());

        // ensure this fn decl's parameters and signature lines up with the one
        // in the trait
        if fn_decl.parameters.len() != fn_signature.parameters.len() {
            errors.push(
                CompileError::IncorrectNumberOfInterfaceSurfaceFunctionParameters {
                    span: fn_decl.parameters_span(),
                    fn_name: fn_decl.name.clone(),
                    interface_name: interface_name(),
                    num_parameters: fn_signature.parameters.len(),
                    provided_parameters: fn_decl.parameters.len(),
                },
            );
            continue;
        }

        // unify the types from the parameters of the function declaration
        // with the parameters of the function signature
        for (fn_signature_param, fn_decl_param) in
            fn_signature.parameters.iter().zip(&fn_decl.parameters)
        {
            // TODO use trait constraints as part of the type here to
            // implement trait constraint solver */
            // check if the mutability of the parameters is incompatible
            if fn_decl_param.is_mutable != fn_signature_param.is_mutable {
                errors.push(CompileError::ParameterMutabilityMismatch {
                    span: fn_decl_param.mutability_span.clone(),
                });
            }

            if (fn_decl_param.is_reference || fn_signature_param.is_reference) && is_contract {
                errors.push(CompileError::RefMutParameterInContract {
                    span: fn_decl_param.mutability_span.clone(),
                });
            }

            let (new_warnings, new_errors) = unify_right_with_self(
                fn_decl_param.type_id,
                fn_signature_param.type_id,
                ctx.self_type(),
                &fn_signature_param.type_span,
                ctx.help_text(),
            );
            if !new_warnings.is_empty() || !new_errors.is_empty() {
                errors.push(CompileError::MismatchedTypeInInterfaceSurface {
                    interface_name: interface_name(),
                    span: fn_decl_param.type_span.clone(),
                    given: fn_decl_param.type_id.to_string(),
                    expected: fn_signature_param.type_id.to_string(),
                });
                continue;
            }
        }

        // check to see if the purity of the function declaration is the same
        // as the purity of the function signature
        if fn_decl.purity != fn_signature.purity {
            errors.push(if fn_signature.purity == Purity::Pure {
                CompileError::TraitDeclPureImplImpure {
                    fn_name: fn_decl.name.clone(),
                    interface_name: interface_name(),
                    attrs: fn_decl.purity.to_attribute_syntax(),
                    span: fn_decl.span.clone(),
                }
            } else {
                CompileError::TraitImplPurityMismatch {
                    fn_name: fn_decl.name.clone(),
                    interface_name: interface_name(),
                    attrs: fn_signature.purity.to_attribute_syntax(),
                    span: fn_decl.span.clone(),
                }
            });
        }

        // unify the return type of the implemented function and the return
        // type of the signature
        let (new_warnings, new_errors) = unify_right_with_self(
            fn_decl.return_type,
            fn_signature.return_type,
            ctx.self_type(),
            &fn_decl.return_type_span,
            ctx.help_text(),
        );
        if !new_warnings.is_empty() || !new_errors.is_empty() {
            errors.push(CompileError::MismatchedTypeInInterfaceSurface {
                interface_name: interface_name(),
                span: fn_decl.return_type_span.clone(),
                expected: fn_signature.return_type.to_string(),
                given: fn_decl.return_type.to_string(),
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
        let unconstrained_type_parameters_in_this_function: HashSet<TypeParameter> = fn_decl
            .unconstrained_type_parameters(impl_type_parameters)
            .into_iter()
            .cloned()
            .collect();
        let unconstrained_type_parameters_in_the_type: HashSet<TypeParameter> =
            type_implementing_for
                .unconstrained_type_parameters(impl_type_parameters)
                .into_iter()
                .cloned()
                .collect::<HashSet<_>>();
        let mut unconstrained_type_parameters_to_be_added =
            unconstrained_type_parameters_in_this_function
                .difference(&unconstrained_type_parameters_in_the_type)
                .cloned()
                .into_iter()
                .collect::<Vec<_>>();
        fn_decl
            .type_parameters
            .append(&mut unconstrained_type_parameters_to_be_added);

        functions_buf.push(fn_decl);
    }

    // This name space is temporary! It is used only so that the below methods
    // can reference functions from the interface
    let mut impl_trait_namespace = ctx.namespace.clone();
    let ctx = ctx.scoped(&mut impl_trait_namespace);

    // A trait impl needs access to everything that the trait methods have access to, which is
    // basically everything in the path where the trait is declared.
    // First, get the path to where the trait is declared. This is a combination of the path stored
    // in the symbols map and the path stored in the CallPath.
    let trait_path = [
        &trait_name.prefixes[..],
        ctx.namespace.get_canonical_path(&trait_name.suffix),
    ]
    .concat();
    ctx.namespace.star_import(&trait_path);

    let self_type_id = insert_type(
        match to_typeinfo(ctx.self_type(), type_implementing_for_span) {
            Ok(o) => o,
            Err(e) => {
                errors.push(e.into());
                return err(warnings, errors);
            }
        },
    );
    check!(
        ctx.namespace.insert_trait_implementation(
            CallPath {
                prefixes: vec![],
                suffix: trait_name.suffix.clone(),
                is_absolute: false,
            },
            trait_type_arguments.to_vec(),
            self_type_id,
            functions_buf.clone(),
            block_span,
            false,
        ),
        return err(warnings, errors),
        warnings,
        errors
    );

    let mut ctx = ctx
        .with_help_text("")
        .with_type_annotation(insert_type(TypeInfo::Unknown));

    // type check the methods now that the interface
    // they depends upon has been implemented
    // use a local namespace which has the above interface inserted
    // into it as a trait implementation for this
    for method in trait_methods {
        let method = check!(
            ty::TyFunctionDeclaration::type_check(ctx.by_ref(), method.clone(), true),
            continue,
            warnings,
            errors
        );
        functions_buf.push(method);
    }

    // check that the implementation checklist is complete
    if !function_checklist.is_empty() {
        errors.push(CompileError::MissingInterfaceSurfaceMethods {
            span: block_span.clone(),
            missing_functions: function_checklist
                .into_iter()
                .map(|(ident, _)| ident.as_str().to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        });
    }
    ok(functions_buf, warnings, errors)
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
            .map(|x| (look_up_type_id(x.type_id), x.span())),
    );

    // create a list of the generics in use in the impl signature
    let mut generics_in_use = HashSet::new();
    for type_arg in trait_type_arguments.iter() {
        generics_in_use.extend(check!(
            look_up_type_id(type_arg.type_id).extract_nested_generics(&type_arg.span),
            HashSet::new(),
            warnings,
            errors
        ));
    }
    generics_in_use.extend(check!(
        look_up_type_id(self_type).extract_nested_generics(self_type_span),
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
            ty: format!("{}", k),
            span: v,
        });
    }

    if errors.is_empty() {
        ok((), warnings, errors)
    } else {
        err(warnings, errors)
    }
}
