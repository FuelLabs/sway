impl TyAstNode {
    /// Returns `true` if this AST node will be exported in a library, i.e. it is a public declaration.
    pub(crate) fn is_public(&self) -> CompileResult<bool> {
        use TyAstNodeContent::*;
        let mut warnings = vec![];
        let mut errors = vec![];
        let public = match &self.content {
            Declaration(decl) => {
                let visibility = check!(
                    decl.visibility(),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                visibility.is_public()
            }
            Expression(_) | SideEffect | ImplicitReturnExpression(_) => false,
        };
        ok(public, warnings, errors)
    }

    /// Naive check to see if this node is a function declaration of a function called `main` if
    /// the [TreeType] is Script or Predicate.
    pub(crate) fn is_main_function(&self, tree_type: TreeType) -> CompileResult<bool> {
        let mut warnings = vec![];
        let mut errors = vec![];
        match &self {
            TyAstNode {
                span,
                content: TyAstNodeContent::Declaration(TyDeclaration::FunctionDeclaration(decl_id)),
                ..
            } => {
                let TyFunctionDeclaration { name, .. } = check!(
                    CompileResult::from(de_get_function(decl_id.clone(), span)),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let is_main = name.as_str() == crate::constants::DEFAULT_ENTRY_POINT_FN_NAME
                    && matches!(tree_type, TreeType::Script | TreeType::Predicate);
                ok(is_main, warnings, errors)
            }
            _ => ok(false, warnings, errors),
        }
    }

    /// recurse into `self` and get any return statements -- used to validate that all returns
    /// do indeed return the correct type
    /// This does _not_ extract implicit return statements as those are not control flow! This is
    /// _only_ for explicit returns.
    pub(crate) fn gather_return_statements(&self) -> Vec<&TyReturnStatement> {
        match &self.content {
            TyAstNodeContent::ImplicitReturnExpression(ref exp) => exp.gather_return_statements(),
            // assignments and  reassignments can happen during control flow and can abort
            TyAstNodeContent::Declaration(TyDeclaration::VariableDeclaration(decl)) => {
                decl.body.gather_return_statements()
            }
            TyAstNodeContent::Expression(exp) => exp.gather_return_statements(),
            TyAstNodeContent::SideEffect | TyAstNodeContent::Declaration(_) => vec![],
        }
    }

    fn type_info(&self) -> TypeInfo {
        // return statement should be ()
        use TyAstNodeContent::*;
        match &self.content {
            Declaration(_) => TypeInfo::Tuple(Vec::new()),
            Expression(TyExpression { return_type, .. }) => {
                crate::type_system::look_up_type_id(*return_type)
            }
            ImplicitReturnExpression(TyExpression { return_type, .. }) => {
                crate::type_system::look_up_type_id(*return_type)
            }
            SideEffect => TypeInfo::Tuple(Vec::new()),
        }
    }

    pub(crate) fn type_check(mut ctx: TypeCheckContext, node: AstNode) -> CompileResult<Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // A little utility used to check an ascribed type matches its associated expression.
        let mut type_check_ascribed_expr =
            |mut ctx: TypeCheckContext, type_ascription: TypeInfo, expr| {
                let type_id = check!(
                    ctx.resolve_type_with_self(
                        insert_type(type_ascription),
                        &node.span,
                        EnforceTypeArguments::No,
                        None
                    ),
                    insert_type(TypeInfo::ErrorRecovery),
                    warnings,
                    errors,
                );
                let ctx = ctx.with_type_annotation(type_id).with_help_text(
                    "This declaration's type annotation does not match up with the assigned \
                        expression's type.",
                );
                TyExpression::type_check(ctx, expr)
            };

        let node = TyAstNode {
            content: match node.content.clone() {
                AstNodeContent::UseStatement(a) => {
                    let path = if a.is_absolute {
                        a.call_path.clone()
                    } else {
                        ctx.namespace.find_module_path(&a.call_path)
                    };
                    let mut res = match a.import_type {
                        ImportType::Star => ctx.namespace.star_import(&path),
                        ImportType::SelfImport => ctx.namespace.self_import(&path, a.alias),
                        ImportType::Item(s) => ctx.namespace.item_import(&path, &s, a.alias),
                    };
                    warnings.append(&mut res.warnings);
                    errors.append(&mut res.errors);
                    TyAstNodeContent::SideEffect
                }
                AstNodeContent::IncludeStatement(_) => TyAstNodeContent::SideEffect,
                AstNodeContent::Declaration(a) => {
                    TyAstNodeContent::Declaration(match a {
                        Declaration::VariableDeclaration(VariableDeclaration {
                            name,
                            type_ascription,
                            type_ascription_span,
                            body,
                            is_mutable,
                        }) => {
                            let type_ascription = check!(
                                ctx.resolve_type_with_self(
                                    insert_type(type_ascription),
                                    &type_ascription_span.clone().unwrap_or_else(|| name.span()),
                                    EnforceTypeArguments::Yes,
                                    None
                                ),
                                insert_type(TypeInfo::ErrorRecovery),
                                warnings,
                                errors
                            );
                            let mut ctx = ctx.with_type_annotation(type_ascription).with_help_text(
                                "Variable declaration's type annotation does not match up \
                                    with the assigned expression's type.",
                            );
                            let result = TyExpression::type_check(ctx.by_ref(), body);
                            let body =
                                check!(result, error_recovery_expr(name.span()), warnings, errors);
                            let typed_var_decl = TyDeclaration::VariableDeclaration(Box::new(
                                TyVariableDeclaration {
                                    name: name.clone(),
                                    body,
                                    mutability: convert_to_variable_immutability(false, is_mutable),
                                    type_ascription,
                                    type_ascription_span,
                                },
                            ));
                            ctx.namespace.insert_symbol(name, typed_var_decl.clone());
                            typed_var_decl
                        }
                        Declaration::ConstantDeclaration(ConstantDeclaration {
                            name,
                            type_ascription,
                            value,
                            visibility,
                            ..
                        }) => {
                            let result =
                                type_check_ascribed_expr(ctx.by_ref(), type_ascription, value);
                            is_screaming_snake_case(&name).ok(&mut warnings, &mut errors);
                            let value =
                                check!(result, error_recovery_expr(name.span()), warnings, errors);
                            let decl = TyConstantDeclaration {
                                name: name.clone(),
                                value,
                                visibility,
                            };
                            let typed_const_decl =
                                TyDeclaration::ConstantDeclaration(de_insert_constant(decl));
                            ctx.namespace.insert_symbol(name, typed_const_decl.clone());
                            typed_const_decl
                        }
                        Declaration::EnumDeclaration(decl) => {
                            let enum_decl = check!(
                                TyEnumDeclaration::type_check(ctx.by_ref(), decl),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let name = enum_decl.name.clone();
                            let decl = TyDeclaration::EnumDeclaration(de_insert_enum(enum_decl));
                            check!(
                                ctx.namespace.insert_symbol(name, decl.clone()),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            decl
                        }
                        Declaration::FunctionDeclaration(fn_decl) => {
                            let mut ctx = ctx.with_type_annotation(insert_type(TypeInfo::Unknown));
                            let fn_decl = check!(
                                TyFunctionDeclaration::type_check(ctx.by_ref(), fn_decl.clone()),
                                error_recovery_function_declaration(fn_decl),
                                warnings,
                                errors
                            );

                            let name = fn_decl.name.clone();
                            let decl =
                                TyDeclaration::FunctionDeclaration(de_insert_function(fn_decl));
                            ctx.namespace.insert_symbol(name, decl.clone());
                            decl
                        }
                        Declaration::TraitDeclaration(trait_decl) => {
                            let trait_decl = check!(
                                TyTraitDeclaration::type_check(ctx.by_ref(), trait_decl),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let name = trait_decl.name.clone();
                            let decl_id = de_insert_trait(trait_decl);
                            let decl = TyDeclaration::TraitDeclaration(decl_id);
                            ctx.namespace.insert_symbol(name, decl.clone());
                            decl
                        }
                        Declaration::ImplTrait(impl_trait) => {
                            let (impl_trait, implementing_for_type_id) = check!(
                                TyImplTrait::type_check_impl_trait(ctx.by_ref(), impl_trait),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            ctx.namespace.insert_trait_implementation(
                                impl_trait.trait_name.clone(),
                                implementing_for_type_id,
                                impl_trait.methods.clone(),
                            );
                            TyDeclaration::ImplTrait(de_insert_impl_trait(impl_trait))
                        }
                        Declaration::ImplSelf(impl_self) => {
                            let impl_trait = check!(
                                TyImplTrait::type_check_impl_self(ctx.by_ref(), impl_self),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            ctx.namespace.insert_trait_implementation(
                                impl_trait.trait_name.clone(),
                                impl_trait.implementing_for_type_id,
                                impl_trait.methods.clone(),
                            );
                            TyDeclaration::ImplTrait(de_insert_impl_trait(impl_trait))
                        }
                        Declaration::StructDeclaration(decl) => {
                            let decl = check!(
                                TyStructDeclaration::type_check(ctx.by_ref(), decl),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let name = decl.name.clone();
                            let decl_id = de_insert_struct(decl);
                            let decl = TyDeclaration::StructDeclaration(decl_id);
                            // insert the struct decl into namespace
                            check!(
                                ctx.namespace.insert_symbol(name, decl.clone()),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            decl
                        }
                        Declaration::AbiDeclaration(abi_decl) => {
                            let abi_decl = check!(
                                TyAbiDeclaration::type_check(ctx.by_ref(), abi_decl),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let name = abi_decl.name.clone();
                            let decl = TyDeclaration::AbiDeclaration(de_insert_abi(abi_decl));
                            ctx.namespace.insert_symbol(name, decl.clone());
                            decl
                        }
                        Declaration::StorageDeclaration(StorageDeclaration {
                            span,
                            fields,
                            ..
                        }) => {
                            let mut fields_buf = Vec::with_capacity(fields.len());
                            for StorageField {
                                name,
                                type_info,
                                initializer,
                                type_info_span,
                                ..
                            } in fields
                            {
                                let type_id = check!(
                                    ctx.resolve_type_without_self(
                                        insert_type(type_info),
                                        &name.span(),
                                        None
                                    ),
                                    return err(warnings, errors),
                                    warnings,
                                    errors
                                );

                                let mut ctx = ctx.by_ref().with_type_annotation(type_id);
                                let initializer = check!(
                                    TyExpression::type_check(ctx.by_ref(), initializer),
                                    return err(warnings, errors),
                                    warnings,
                                    errors,
                                );

                                fields_buf.push(TyStorageField::new(
                                    name,
                                    type_id,
                                    type_info_span,
                                    initializer,
                                    span.clone(),
                                ));
                            }
                            let decl = TyStorageDeclaration::new(fields_buf, span);
                            let decl_id = de_insert_storage(decl);
                            // insert the storage declaration into the symbols
                            // if there already was one, return an error that duplicate storage

                            // declarations are not allowed
                            check!(
                                ctx.namespace.set_storage_declaration(decl_id.clone()),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            TyDeclaration::StorageDeclaration(decl_id)
                        }
                    })
                }
                AstNodeContent::Expression(expr) => {
                    let ctx = ctx
                        .with_type_annotation(insert_type(TypeInfo::Unknown))
                        .with_help_text("");
                    let inner = check!(
                        TyExpression::type_check(ctx, expr.clone()),
                        error_recovery_expr(expr.span()),
                        warnings,
                        errors
                    );
                    TyAstNodeContent::Expression(inner)
                }
                AstNodeContent::ImplicitReturnExpression(expr) => {
                    let ctx =
                        ctx.with_help_text("Implicit return must match up with block's type.");
                    let typed_expr = check!(
                        TyExpression::type_check(ctx, expr.clone()),
                        error_recovery_expr(expr.span()),
                        warnings,
                        errors
                    );
                    TyAstNodeContent::ImplicitReturnExpression(typed_expr)
                }
            },
            span: node.span.clone(),
        };

        if let TyAstNode {
            content: TyAstNodeContent::Expression(TyExpression { .. }),
            ..
        } = node
        {
            let warning = Warning::UnusedReturnValue {
                r#type: Box::new(node.type_info()),
            };
            assert_or_warn!(
                node.type_info().can_safely_ignore(),
                warnings,
                node.span.clone(),
                warning
            );
        }

        ok(node, warnings, errors)
    }
}

fn type_check_interface_surface(
    interface_surface: Vec<TraitFn>,
    namespace: &mut Namespace,
) -> CompileResult<Vec<TyTraitFn>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut typed_surface = vec![];
    for trait_fn in interface_surface.into_iter() {
        let TraitFn {
            name,
            purity,
            parameters,
            return_type,
            return_type_span,
            ..
        } = trait_fn;

        // type check the parameters
        let mut typed_parameters = vec![];
        for param in parameters.into_iter() {
            typed_parameters.push(check!(
                TyFunctionParameter::type_check_interface_parameter(namespace, param),
                continue,
                warnings,
                errors
            ));
        }

        // type check the return type
        let return_type = check!(
            namespace.resolve_type_with_self(
                insert_type(return_type),
                insert_type(TypeInfo::SelfType),
                &return_type_span,
                EnforceTypeArguments::Yes,
                None
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );

        typed_surface.push(TyTraitFn {
            name,
            purity,
            return_type_span,
            parameters: typed_parameters,
            return_type,
        });
    }
    ok(typed_surface, warnings, errors)
}

fn type_check_trait_methods(
    mut ctx: TypeCheckContext,
    methods: Vec<FunctionDeclaration>,
) -> CompileResult<Vec<TyFunctionDeclaration>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut methods_buf = Vec::new();
    for method in methods.into_iter() {
        let FunctionDeclaration {
            body,
            name: fn_name,
            parameters,
            span,
            return_type,
            type_parameters,
            return_type_span,
            purity,
            ..
        } = method;

        // A context while checking the signature where `self_type` refers to `SelfType`.
        let mut sig_ctx = ctx.by_ref().with_self_type(insert_type(TypeInfo::SelfType));

        // type check the function parameters
        // which will also insert them into the namespace
        let mut typed_parameters = vec![];
        for parameter in parameters.into_iter() {
            typed_parameters.push(check!(
                TyFunctionParameter::type_check_method_parameter(sig_ctx.by_ref(), parameter),
                continue,
                warnings,
                errors
            ));
        }

        // check the generic types in the arguments, make sure they are in
        // the type scope
        let mut generic_params_buf_for_error_message = Vec::new();
        for param in typed_parameters.iter() {
            if let TypeInfo::Custom { ref name, .. } = look_up_type_id(param.type_id) {
                generic_params_buf_for_error_message.push(name.to_string());
            }
        }
        let comma_separated_generic_params = generic_params_buf_for_error_message.join(", ");
        for param in typed_parameters.iter() {
            let span = param.name.span().clone();
            if let TypeInfo::Custom { name, .. } = look_up_type_id(param.type_id) {
                let args_span = typed_parameters.iter().fold(
                    typed_parameters[0].name.span().clone(),
                    |acc, TyFunctionParameter { name, .. }| Span::join(acc, name.span()),
                );
                if type_parameters.iter().any(|TypeParameter { type_id, .. }| {
                    if let TypeInfo::Custom {
                        name: this_name, ..
                    } = look_up_type_id(*type_id)
                    {
                        this_name == name.clone()
                    } else {
                        false
                    }
                }) {
                    errors.push(CompileError::TypeParameterNotInTypeScope {
                        name: name.clone(),
                        span: span.clone(),
                        comma_separated_generic_params: comma_separated_generic_params.clone(),
                        fn_name: fn_name.clone(),
                        args: args_span.as_str().to_string(),
                    });
                }
            }
        }

        // type check the return type
        // TODO check code block implicit return
        let initial_return_type = insert_type(return_type);
        let return_type = check!(
            ctx.resolve_type_with_self(
                initial_return_type,
                &return_type_span,
                EnforceTypeArguments::Yes,
                None
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );

        // type check the body
        let ctx = ctx
            .by_ref()
            .with_purity(purity)
            .with_type_annotation(return_type)
            .with_help_text(
                "Trait method body's return type does not match up with its return type \
                annotation.",
            );
        let (body, _code_block_implicit_return) = check!(
            TyCodeBlock::type_check(ctx, body),
            continue,
            warnings,
            errors
        );

        methods_buf.push(TyFunctionDeclaration {
            name: fn_name,
            body,
            parameters: typed_parameters,
            span,
            return_type,
            initial_return_type,
            type_parameters,
            // For now, any method declared is automatically public.
            // We can tweak that later if we want.
            visibility: Visibility::Public,
            return_type_span,
            is_contract_call: false,
            purity,
        });
    }
    ok(methods_buf, warnings, errors)
}

/// Used to create a stubbed out function when the function fails to compile, preventing cascading
/// namespace errors
fn error_recovery_function_declaration(decl: FunctionDeclaration) -> TyFunctionDeclaration {
    let FunctionDeclaration {
        name,
        return_type,
        span,
        return_type_span,
        visibility,
        ..
    } = decl;
    let initial_return_type = insert_type(return_type);
    TyFunctionDeclaration {
        purity: Default::default(),
        name,
        body: TyCodeBlock {
            contents: Default::default(),
        },
        span,
        is_contract_call: false,
        return_type_span,
        parameters: Default::default(),
        visibility,
        return_type: initial_return_type,
        initial_return_type,
        type_parameters: Default::default(),
    }
}
