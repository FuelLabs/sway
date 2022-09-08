pub mod code_block;
pub mod declaration;
pub mod expression;
pub mod mode;
mod return_statement;

use std::fmt;

pub(crate) use code_block::*;
pub use declaration::*;
pub(crate) use expression::*;
pub(crate) use mode::*;
pub(crate) use return_statement::*;

use crate::{
    declaration_engine::declaration_engine::*, error::*, parse_tree::*, semantic_analysis::*,
    style::*, type_system::*, types::DeterministicallyAborts, AstNode, AstNodeContent, Ident,
    ReturnStatement,
};

use sway_types::{span::Span, state::StateIndex, Spanned};

use derivative::Derivative;

/// whether or not something is constantly evaluatable (if the result is known at compile
/// time)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum IsConstant {
    Yes,
    No,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedAstNodeContent {
    ReturnStatement(TypedReturnStatement),
    Declaration(TypedDeclaration),
    Expression(TypedExpression),
    ImplicitReturnExpression(TypedExpression),
    // a no-op node used for something that just issues a side effect, like an import statement.
    SideEffect,
}

impl CollectTypesMetadata for TypedAstNodeContent {
    fn collect_types_metadata(&self) -> CompileResult<Vec<TypeMetadata>> {
        use TypedAstNodeContent::*;
        match self {
            ReturnStatement(stmt) => stmt.expr.collect_types_metadata(),
            Declaration(decl) => decl.collect_types_metadata(),
            Expression(expr) => expr.collect_types_metadata(),
            ImplicitReturnExpression(expr) => expr.collect_types_metadata(),
            SideEffect => ok(vec![], vec![], vec![]),
        }
    }
}

#[derive(Clone, Debug, Eq, Derivative)]
#[derivative(PartialEq)]
pub struct TypedAstNode {
    pub content: TypedAstNodeContent,
    #[derivative(PartialEq = "ignore")]
    pub(crate) span: Span,
}

impl fmt::Display for TypedAstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TypedAstNodeContent::*;
        let text = match &self.content {
            ReturnStatement(TypedReturnStatement { ref expr }) => {
                format!("return {}", expr)
            }
            Declaration(ref typed_decl) => typed_decl.to_string(),
            Expression(exp) => exp.to_string(),
            ImplicitReturnExpression(exp) => format!("return {}", exp),
            SideEffect => "".into(),
        };
        f.write_str(&text)
    }
}

impl CopyTypes for TypedAstNode {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        match self.content {
            TypedAstNodeContent::ReturnStatement(ref mut ret_stmt) => {
                ret_stmt.copy_types(type_mapping)
            }
            TypedAstNodeContent::ImplicitReturnExpression(ref mut exp) => {
                exp.copy_types(type_mapping)
            }
            TypedAstNodeContent::Declaration(ref mut decl) => decl.copy_types(type_mapping),
            TypedAstNodeContent::Expression(ref mut expr) => expr.copy_types(type_mapping),
            TypedAstNodeContent::SideEffect => (),
        }
    }
}

impl CollectTypesMetadata for TypedAstNode {
    fn collect_types_metadata(&self) -> CompileResult<Vec<TypeMetadata>> {
        self.content.collect_types_metadata()
    }
}

impl DeterministicallyAborts for TypedAstNode {
    fn deterministically_aborts(&self) -> bool {
        use TypedAstNodeContent::*;
        match &self.content {
            ReturnStatement(_) => true,
            Declaration(_) => false,
            Expression(exp) | ImplicitReturnExpression(exp) => exp.deterministically_aborts(),
            SideEffect => false,
        }
    }
}

impl TypedAstNode {
    /// Returns `true` if this AST node will be exported in a library, i.e. it is a public declaration.
    pub(crate) fn is_public(&self) -> CompileResult<bool> {
        use TypedAstNodeContent::*;
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
            ReturnStatement(_) | Expression(_) | SideEffect | ImplicitReturnExpression(_) => false,
        };
        ok(public, warnings, errors)
    }

    /// Naive check to see if this node is a function declaration of a function called `main` if
    /// the [TreeType] is Script or Predicate.
    pub(crate) fn is_main_function(&self, tree_type: TreeType) -> bool {
        match &self {
            TypedAstNode {
                content:
                    TypedAstNodeContent::Declaration(TypedDeclaration::FunctionDeclaration(
                        TypedFunctionDeclaration { name, .. },
                    )),
                ..
            } if name.as_str() == crate::constants::DEFAULT_ENTRY_POINT_FN_NAME => {
                matches!(tree_type, TreeType::Script | TreeType::Predicate)
            }
            _ => false,
        }
    }

    /// recurse into `self` and get any return statements -- used to validate that all returns
    /// do indeed return the correct type
    /// This does _not_ extract implicit return statements as those are not control flow! This is
    /// _only_ for explicit returns.
    pub(crate) fn gather_return_statements(&self) -> Vec<&TypedReturnStatement> {
        match &self.content {
            TypedAstNodeContent::ReturnStatement(ref stmt) => vec![stmt],
            TypedAstNodeContent::ImplicitReturnExpression(ref exp) => {
                exp.gather_return_statements()
            }
            // assignments and  reassignments can happen during control flow and can abort
            TypedAstNodeContent::Declaration(TypedDeclaration::VariableDeclaration(decl)) => {
                decl.body.gather_return_statements()
            }
            TypedAstNodeContent::Expression(exp) => exp.gather_return_statements(),
            TypedAstNodeContent::SideEffect | TypedAstNodeContent::Declaration(_) => vec![],
        }
    }

    fn type_info(&self) -> TypeInfo {
        // return statement should be ()
        use TypedAstNodeContent::*;
        match &self.content {
            ReturnStatement(_) | Declaration(_) => TypeInfo::Tuple(Vec::new()),
            Expression(TypedExpression { return_type, .. }) => {
                crate::type_system::look_up_type_id(*return_type)
            }
            ImplicitReturnExpression(TypedExpression { return_type, .. }) => {
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
                TypedExpression::type_check(ctx, expr)
            };

        let node = TypedAstNode {
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
                    TypedAstNodeContent::SideEffect
                }
                AstNodeContent::IncludeStatement(_) => TypedAstNodeContent::SideEffect,
                AstNodeContent::Declaration(a) => {
                    TypedAstNodeContent::Declaration(match a {
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
                            let result = TypedExpression::type_check(ctx.by_ref(), body);
                            let body =
                                check!(result, error_recovery_expr(name.span()), warnings, errors);
                            let typed_var_decl = TypedDeclaration::VariableDeclaration(Box::new(
                                TypedVariableDeclaration {
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
                            let decl = TypedConstantDeclaration {
                                name: name.clone(),
                                value,
                                visibility,
                            };
                            let typed_const_decl =
                                TypedDeclaration::ConstantDeclaration(de_insert_constant(decl));
                            ctx.namespace.insert_symbol(name, typed_const_decl.clone());
                            typed_const_decl
                        }
                        Declaration::EnumDeclaration(decl) => {
                            let enum_decl = check!(
                                TypedEnumDeclaration::type_check(ctx.by_ref(), decl),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let name = enum_decl.name.clone();
                            let decl = TypedDeclaration::EnumDeclaration(enum_decl);
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
                                TypedFunctionDeclaration::type_check(ctx.by_ref(), fn_decl.clone()),
                                error_recovery_function_declaration(fn_decl),
                                warnings,
                                errors
                            );
                            let name = fn_decl.name.clone();
                            let decl = TypedDeclaration::FunctionDeclaration(fn_decl);
                            ctx.namespace.insert_symbol(name, decl.clone());
                            decl
                        }
                        Declaration::TraitDeclaration(trait_decl) => {
                            let trait_decl = check!(
                                TypedTraitDeclaration::type_check(ctx.by_ref(), trait_decl),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let name = trait_decl.name.clone();
                            let decl_id = de_insert_trait(trait_decl);
                            let decl = TypedDeclaration::TraitDeclaration(decl_id);
                            ctx.namespace.insert_symbol(name, decl.clone());
                            decl
                        }
                        Declaration::ImplTrait(impl_trait) => {
                            let (impl_trait, implementing_for_type_id) = check!(
                                TypedImplTrait::type_check_impl_trait(ctx.by_ref(), impl_trait),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            ctx.namespace.insert_trait_implementation(
                                impl_trait.trait_name.clone(),
                                implementing_for_type_id,
                                impl_trait.methods.clone(),
                            );
                            TypedDeclaration::ImplTrait(de_insert_impl_trait(impl_trait))
                        }
                        Declaration::ImplSelf(impl_self) => {
                            let impl_trait = check!(
                                TypedImplTrait::type_check_impl_self(ctx.by_ref(), impl_self),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            ctx.namespace.insert_trait_implementation(
                                impl_trait.trait_name.clone(),
                                impl_trait.implementing_for_type_id,
                                impl_trait.methods.clone(),
                            );
                            TypedDeclaration::ImplTrait(de_insert_impl_trait(impl_trait))
                        }
                        Declaration::StructDeclaration(decl) => {
                            let decl = check!(
                                TypedStructDeclaration::type_check(ctx.by_ref(), decl),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let name = decl.name.clone();
                            let decl = TypedDeclaration::StructDeclaration(decl);
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
                                TypedAbiDeclaration::type_check(ctx.by_ref(), abi_decl),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let name = abi_decl.name.clone();
                            let decl = TypedDeclaration::AbiDeclaration(de_insert_abi(abi_decl));
                            ctx.namespace.insert_symbol(name, decl.clone());
                            decl
                        }
                        Declaration::StorageDeclaration(StorageDeclaration { span, fields }) => {
                            let mut fields_buf = Vec::with_capacity(fields.len());
                            for StorageField {
                                name,
                                type_info,
                                initializer,
                                type_info_span,
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
                                    TypedExpression::type_check(ctx.by_ref(), initializer),
                                    return err(warnings, errors),
                                    warnings,
                                    errors,
                                );

                                fields_buf.push(TypedStorageField::new(
                                    name,
                                    type_id,
                                    type_info_span,
                                    initializer,
                                    span.clone(),
                                ));
                            }
                            let decl = TypedStorageDeclaration::new(fields_buf, span);
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
                            TypedDeclaration::StorageDeclaration(decl_id)
                        }
                    })
                }
                AstNodeContent::Expression(expr) => {
                    let ctx = ctx
                        .with_type_annotation(insert_type(TypeInfo::Unknown))
                        .with_help_text("");
                    let inner = check!(
                        TypedExpression::type_check(ctx, expr.clone()),
                        error_recovery_expr(expr.span()),
                        warnings,
                        errors
                    );
                    TypedAstNodeContent::Expression(inner)
                }
                AstNodeContent::ReturnStatement(ReturnStatement { expr }) => {
                    let ctx = ctx
                        // we use "unknown" here because return statements do not
                        // necessarily follow the type annotation of their immediate
                        // surrounding context. Because a return statement is control flow
                        // that breaks out to the nearest function, we need to type check
                        // it against the surrounding function.
                        // That is impossible here, as we don't have that information. It
                        // is the responsibility of the function declaration to type check
                        // all return statements contained within it.
                        .with_type_annotation(insert_type(TypeInfo::Unknown))
                        .with_help_text(
                            "Returned value must match up with the function return type \
                            annotation.",
                        );

                    TypedAstNodeContent::ReturnStatement(TypedReturnStatement {
                        expr: check!(
                            TypedExpression::type_check(ctx, expr.clone()),
                            error_recovery_expr(expr.span()),
                            warnings,
                            errors
                        ),
                    })
                }
                AstNodeContent::ImplicitReturnExpression(expr) => {
                    let ctx =
                        ctx.with_help_text("Implicit return must match up with block's type.");
                    let typed_expr = check!(
                        TypedExpression::type_check(ctx, expr.clone()),
                        error_recovery_expr(expr.span()),
                        warnings,
                        errors
                    );
                    TypedAstNodeContent::ImplicitReturnExpression(typed_expr)
                }
            },
            span: node.span.clone(),
        };

        if let TypedAstNode {
            content: TypedAstNodeContent::Expression(TypedExpression { ref expression, .. }),
            ..
        } = node
        {
            if !matches!(
                expression,
                TypedExpressionVariant::Break | TypedExpressionVariant::Continue,
            ) {
                let warning = Warning::UnusedReturnValue {
                    r#type: Box::new(node.type_info()),
                };
                assert_or_warn!(
                    node.type_info().is_unit() || node.type_info() == TypeInfo::ErrorRecovery,
                    warnings,
                    node.span.clone(),
                    warning
                );
            }
        }

        ok(node, warnings, errors)
    }
}

fn type_check_interface_surface(
    interface_surface: Vec<TraitFn>,
    namespace: &mut Namespace,
) -> CompileResult<Vec<TypedTraitFn>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let interface_surface = interface_surface
        .into_iter()
        .map(
            |TraitFn {
                 name,
                 purity,
                 parameters,
                 return_type,
                 return_type_span,
             }| TypedTraitFn {
                name,
                purity,
                return_type_span: return_type_span.clone(),
                parameters: parameters
                    .into_iter()
                    .map(
                        |FunctionParameter {
                             name,
                             is_reference,
                             is_mutable,
                             type_id,
                             type_span,
                         }| TypedFunctionParameter {
                            name,
                            is_reference,
                            is_mutable,
                            type_id: check!(
                                namespace.resolve_type_with_self(
                                    type_id,
                                    insert_type(TypeInfo::SelfType),
                                    &type_span,
                                    EnforceTypeArguments::Yes,
                                    None
                                ),
                                insert_type(TypeInfo::ErrorRecovery),
                                warnings,
                                errors,
                            ),
                            initial_type_id: type_id,
                            type_span,
                        },
                    )
                    .collect(),
                return_type: check!(
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
                ),
            },
        )
        .collect::<Vec<_>>();
    ok(interface_surface, warnings, errors)
}

fn type_check_trait_methods(
    mut ctx: TypeCheckContext,
    methods: Vec<FunctionDeclaration>,
) -> CompileResult<Vec<TypedFunctionDeclaration>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut methods_buf = Vec::new();
    for FunctionDeclaration {
        body,
        name: fn_name,
        parameters,
        span,
        return_type,
        type_parameters,
        return_type_span,
        purity,
        ..
    } in methods
    {
        // A context while checking the signature where `self_type` refers to `SelfType`.
        let mut sig_ctx = ctx.by_ref().with_self_type(insert_type(TypeInfo::SelfType));
        parameters.clone().into_iter().for_each(
            |FunctionParameter {
                 name,
                 is_reference,
                 is_mutable,
                 ref type_id,
                 ..
             }| {
                let r#type = check!(
                    sig_ctx.resolve_type_with_self(
                        *type_id,
                        &name.span(),
                        EnforceTypeArguments::Yes,
                        None
                    ),
                    insert_type(TypeInfo::ErrorRecovery),
                    warnings,
                    errors,
                );
                sig_ctx.namespace.insert_symbol(
                    name.clone(),
                    TypedDeclaration::VariableDeclaration(Box::new(TypedVariableDeclaration {
                        name: name.clone(),
                        body: TypedExpression {
                            expression: TypedExpressionVariant::FunctionParameter,
                            return_type: r#type,
                            is_constant: IsConstant::No,
                            span: name.span(),
                        },
                        mutability: convert_to_variable_immutability(is_reference, is_mutable),
                        type_ascription: r#type,
                        type_ascription_span: None,
                    })),
                );
            },
        );
        // check the generic types in the arguments, make sure they are in
        // the type scope
        let mut generic_params_buf_for_error_message = Vec::new();
        for param in parameters.iter() {
            if let TypeInfo::Custom { ref name, .. } = look_up_type_id(param.type_id) {
                generic_params_buf_for_error_message.push(name.to_string());
            }
        }
        let comma_separated_generic_params = generic_params_buf_for_error_message.join(", ");
        for FunctionParameter {
            ref type_id, name, ..
        } in parameters.iter()
        {
            let span = name.span().clone();
            if let TypeInfo::Custom { name, .. } = look_up_type_id(*type_id) {
                let args_span = parameters.iter().fold(
                    parameters[0].name.span().clone(),
                    |acc, FunctionParameter { name, .. }| Span::join(acc, name.span()),
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
        let parameters = parameters
            .into_iter()
            .map(
                |FunctionParameter {
                     name,
                     type_id,
                     is_reference,
                     is_mutable,
                     type_span,
                 }| {
                    TypedFunctionParameter {
                        name,
                        is_reference,
                        is_mutable,
                        type_id: check!(
                            sig_ctx.resolve_type_with_self(
                                type_id,
                                &type_span,
                                EnforceTypeArguments::Yes,
                                None
                            ),
                            insert_type(TypeInfo::ErrorRecovery),
                            warnings,
                            errors,
                        ),
                        initial_type_id: type_id,
                        type_span,
                    }
                },
            )
            .collect::<Vec<_>>();

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
        let ctx = ctx
            .by_ref()
            .with_purity(purity)
            .with_type_annotation(return_type)
            .with_help_text(
                "Trait method body's return type does not match up with its return type \
                annotation.",
            );
        let (body, _code_block_implicit_return) = check!(
            TypedCodeBlock::type_check(ctx, body),
            continue,
            warnings,
            errors
        );

        methods_buf.push(TypedFunctionDeclaration {
            name: fn_name,
            body,
            parameters,
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
fn error_recovery_function_declaration(decl: FunctionDeclaration) -> TypedFunctionDeclaration {
    let FunctionDeclaration {
        name,
        return_type,
        span,
        return_type_span,
        visibility,
        ..
    } = decl;
    let initial_return_type = insert_type(return_type);
    TypedFunctionDeclaration {
        purity: Default::default(),
        name,
        body: TypedCodeBlock {
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

/// Describes each field being drilled down into in storage and its type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeCheckedStorageReassignment {
    pub fields: Vec<TypeCheckedStorageReassignDescriptor>,
    pub(crate) ix: StateIndex,
    pub rhs: TypedExpression,
}

impl Spanned for TypeCheckedStorageReassignment {
    fn span(&self) -> Span {
        self.fields
            .iter()
            .fold(self.fields[0].span.clone(), |acc, field| {
                Span::join(acc, field.span.clone())
            })
    }
}

impl TypeCheckedStorageReassignment {
    pub fn names(&self) -> Vec<Ident> {
        self.fields
            .iter()
            .map(|f| f.name.clone())
            .collect::<Vec<_>>()
    }
}

/// Describes a single subfield access in the sequence when reassigning to a subfield within
/// storage.
#[derive(Clone, Debug, Eq)]
pub struct TypeCheckedStorageReassignDescriptor {
    pub name: Ident,
    pub type_id: TypeId,
    pub(crate) span: Span,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypeCheckedStorageReassignDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && look_up_type_id(self.type_id) == look_up_type_id(other.type_id)
    }
}

pub(crate) fn reassign_storage_subfield(
    ctx: TypeCheckContext,
    fields: Vec<Ident>,
    rhs: Expression,
    span: Span,
) -> CompileResult<TypeCheckedStorageReassignment> {
    let mut errors = vec![];
    let mut warnings = vec![];
    if !ctx.namespace.has_storage_declared() {
        errors.push(CompileError::NoDeclaredStorage { span });

        return err(warnings, errors);
    }

    let storage_fields = check!(
        ctx.namespace.get_storage_field_descriptors(&span),
        return err(warnings, errors),
        warnings,
        errors
    );
    let mut type_checked_buf = vec![];
    let mut fields: Vec<_> = fields.into_iter().rev().collect();

    let first_field = fields.pop().expect("guaranteed by grammar");
    let (ix, initial_field_type) = match storage_fields
        .iter()
        .enumerate()
        .find(|(_, TypedStorageField { name, .. })| name == &first_field)
    {
        Some((
            ix,
            TypedStorageField {
                type_id: r#type, ..
            },
        )) => (StateIndex::new(ix), r#type),
        None => {
            errors.push(CompileError::StorageFieldDoesNotExist {
                name: first_field.clone(),
            });
            return err(warnings, errors);
        }
    };

    type_checked_buf.push(TypeCheckedStorageReassignDescriptor {
        name: first_field.clone(),
        type_id: *initial_field_type,
        span: first_field.span(),
    });

    fn update_available_struct_fields(id: TypeId) -> Vec<TypedStructField> {
        match look_up_type_id(id) {
            TypeInfo::Struct { fields, .. } => fields,
            _ => vec![],
        }
    }
    let mut curr_type = *initial_field_type;

    // if the previously iterated type was a struct, put its fields here so we know that,
    // in the case of a subfield, we can type check the that the subfield exists and its type.
    let mut available_struct_fields = update_available_struct_fields(*initial_field_type);

    // get the initial field's type
    // make sure the next field exists in that type
    for field in fields.into_iter().rev() {
        match available_struct_fields
            .iter()
            .find(|x| x.name.as_str() == field.as_str())
        {
            Some(struct_field) => {
                curr_type = struct_field.type_id;
                type_checked_buf.push(TypeCheckedStorageReassignDescriptor {
                    name: field.clone(),
                    type_id: struct_field.type_id,
                    span: field.span().clone(),
                });
                available_struct_fields = update_available_struct_fields(struct_field.type_id);
            }
            None => {
                let available_fields = available_struct_fields
                    .iter()
                    .map(|x| x.name.as_str())
                    .collect::<Vec<_>>();
                errors.push(CompileError::FieldNotFound {
                    field_name: field.clone(),
                    available_fields: available_fields.join(", "),
                    struct_name: type_checked_buf.last().unwrap().name.clone(),
                });
                return err(warnings, errors);
            }
        }
    }
    let ctx = ctx.with_type_annotation(curr_type).with_help_text("");
    let rhs = check!(
        TypedExpression::type_check(ctx, rhs),
        error_recovery_expr(span),
        warnings,
        errors
    );

    ok(
        TypeCheckedStorageReassignment {
            fields: type_checked_buf,
            ix,
            rhs,
        },
        warnings,
        errors,
    )
}
