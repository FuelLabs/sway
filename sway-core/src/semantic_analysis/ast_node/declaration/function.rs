mod function_parameter;

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
    warning::{CompileWarning, Warning},
};

use crate::{
    decl_engine::{DeclId, DeclRefFunction},
    language::{
        parsed::*,
        ty::{self, TyCodeBlock, TyFunctionDecl},
        CallPath, Visibility,
    },
    semantic_analysis::{type_check_context::EnforceTypeArguments, *},
    type_system::*,
};
use sway_types::{style::is_snake_case, Spanned};

impl ty::TyFunctionDecl {
    pub fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        fn_decl: &FunctionDeclaration,
        is_method: bool,
        is_in_impl_self: bool,
        implementing_for_typeid: Option<TypeId>,
    ) -> Result<Self, ErrorEmitted> {
        let mut ty_fn_decl = Self::type_check_signature(
            handler,
            ctx.by_ref(),
            fn_decl,
            is_method,
            is_in_impl_self,
            implementing_for_typeid,
        )?;
        Self::type_check_body(handler, ctx, fn_decl, &mut ty_fn_decl)
    }

    pub fn type_check_signature(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        fn_decl: &FunctionDeclaration,
        is_method: bool,
        is_in_impl_self: bool,
        implementing_for_typeid: Option<TypeId>,
    ) -> Result<Self, ErrorEmitted> {
        let FunctionDeclaration {
            name,
            body: _,
            parameters,
            span,
            attributes,
            type_parameters,
            visibility,
            purity,
            where_clause,
            kind,
            ..
        } = fn_decl;
        let mut return_type = fn_decl.return_type.clone();

        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        // If functions aren't allowed in this location, return an error.
        if ctx.functions_disallowed() {
            return Err(handler.emit_err(CompileError::Unimplemented {
                feature: "Declaring nested functions".to_string(),
                help: vec![],
                span: span.clone(),
            }));
        }

        // Warn against non-snake case function names.
        if !is_snake_case(name.as_str()) {
            handler.emit_warn(CompileWarning {
                span: name.span(),
                warning_content: Warning::NonSnakeCaseFunctionName { name: name.clone() },
            })
        }

        // create a namespace for the function
        ctx.by_ref()
            .with_purity(*purity)
            .with_const_shadowing_mode(ConstShadowingMode::Sequential)
            .disallow_functions()
            .scoped(|mut ctx| {
                // Type check the type parameters.
                let new_type_parameters = TypeParameter::type_check_type_params(
                    handler,
                    ctx.by_ref(),
                    type_parameters.clone(),
                    None,
                )?;

                // type check the function parameters, which will also insert them into the namespace
                let mut new_parameters = vec![];
                handler.scope(|handler| {
                    for parameter in parameters.iter() {
                        new_parameters.push({
                            let param = match ty::TyFunctionParameter::type_check(
                                handler,
                                ctx.by_ref(),
                                parameter.clone(),
                            ) {
                                Ok(val) => val,
                                Err(_) => continue,
                            };
                            param.insert_into_namespace(handler, ctx.by_ref());
                            param
                        });
                    }
                    Ok(())
                })?;

                // type check the return type
                return_type.type_id = ctx
                    .resolve_type(
                        handler,
                        return_type.type_id,
                        &return_type.span,
                        EnforceTypeArguments::Yes,
                        None,
                    )
                    .unwrap_or_else(|err| {
                        type_engine.insert(engines, TypeInfo::ErrorRecovery(err), None)
                    });

                let (visibility, is_contract_call) = if is_method {
                    if is_in_impl_self {
                        (*visibility, false)
                    } else {
                        (Visibility::Public, false)
                    }
                } else {
                    (
                        *visibility,
                        matches!(ctx.abi_mode(), AbiMode::ImplAbiFn(..)),
                    )
                };

                let call_path =
                    CallPath::from(name.clone()).to_fullpath(ctx.engines(), ctx.namespace());

                let function_decl = ty::TyFunctionDecl {
                    name: name.clone(),
                    body: TyCodeBlock::default(),
                    parameters: new_parameters,
                    implementing_type: None,
                    implementing_for_typeid,
                    span: span.clone(),
                    call_path,
                    attributes: attributes.clone(),
                    return_type,
                    type_parameters: new_type_parameters,
                    visibility,
                    is_contract_call,
                    purity: *purity,
                    where_clause: where_clause.clone(),
                    is_trait_method_dummy: false,
                    kind: match kind {
                        FunctionDeclarationKind::Default => ty::TyFunctionDeclKind::Default,
                        FunctionDeclarationKind::Entry => ty::TyFunctionDeclKind::Entry,
                        FunctionDeclarationKind::Test => ty::TyFunctionDeclKind::Test,
                        FunctionDeclarationKind::Main => ty::TyFunctionDeclKind::Main,
                    },
                };

                Ok(function_decl)
            })
    }

    pub fn type_check_body(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        fn_decl: &FunctionDeclaration,
        ty_fn_decl: &mut Self,
    ) -> Result<Self, ErrorEmitted> {
        // create a namespace for the function
        ctx.by_ref()
            .with_purity(ty_fn_decl.purity)
            .with_const_shadowing_mode(ConstShadowingMode::Sequential)
            .disallow_functions()
            .scoped(|mut ctx| {
                let FunctionDeclaration { body, .. } = fn_decl;

                let ty::TyFunctionDecl {
                    parameters,
                    purity,
                    return_type,
                    type_parameters,
                    ..
                } = ty_fn_decl;

                // Insert the previously type checked type parameters into the current namespace.
                for p in type_parameters {
                    p.insert_into_namespace(handler, ctx.by_ref())?;
                }

                // Insert the previously type checked function parameters into the current namespace.
                for p in parameters {
                    p.insert_into_namespace(handler, ctx.by_ref());
                }

                // type check the function body
                //
                // If there are no implicit block returns, then we do not want to type check them, so we
                // stifle the errors. If there _are_ implicit block returns, we want to type_check them.

                let mut ctx = ctx
                    .by_ref()
                    .with_purity(*purity)
                    .with_help_text(
                        "Function body's return type does not match up with its return type annotation.",
                    )
                    .with_type_annotation(return_type.type_id)
                    .with_function_type_annotation(return_type.type_id);

                let body = ty::TyCodeBlock::type_check(handler, ctx.by_ref(), body)
                    .unwrap_or_else(|_err| ty::TyCodeBlock::default());

                ty_fn_decl.body = body;

                return_type.type_id.check_type_parameter_bounds(
                    handler,
                    ctx.by_ref(),
                    &return_type.span,
                    None,
                )?;

                Ok(ty_fn_decl.clone())
            })
    }
}

impl TypeCheckAnalysis for DeclId<TyFunctionDecl> {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        handler.scope(|handler| {
            let node = ctx.get_node_for_fn_decl(self);
            if let Some(node) = node {
                ctx.node_stack.push(node);

                let item_fn = ctx.engines.de().get_function(self);
                let _ = item_fn.type_check_analyze(handler, ctx);

                ctx.node_stack.pop();
            }
            Ok(())
        })
    }
}

impl TypeCheckAnalysis for DeclRefFunction {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        handler.scope(|handler| {
            let node = ctx.get_node_for_fn_decl(self.id());
            if let Some(node) = node {
                ctx.node_stack.push(node);

                let item_fn = ctx.engines.de().get_function(self);
                let _ = item_fn.type_check_analyze(handler, ctx);

                ctx.node_stack.pop();
            }
            Ok(())
        })
    }
}

impl TypeCheckAnalysis for ty::TyFunctionDecl {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        self.body.type_check_analyze(handler, ctx)
    }
}

impl TypeCheckFinalization for ty::TyFunctionDecl {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        handler.scope(|handler| {
            let _ = self.body.type_check_finalize(handler, ctx);
            Ok(())
        })
    }
}

#[test]
fn test_function_selector_behavior() {
    use crate::language::Visibility;
    use crate::Engines;
    use sway_types::{integer_bits::IntegerBits, Ident, Span};

    let engines = Engines::default();
    let handler = Handler::default();
    let decl = ty::TyFunctionDecl {
        purity: Default::default(),
        name: Ident::dummy(),
        implementing_type: None,
        implementing_for_typeid: None,
        body: ty::TyCodeBlock::default(),
        parameters: vec![],
        span: Span::dummy(),
        call_path: CallPath::from(Ident::dummy()),
        attributes: Default::default(),
        return_type: TypeId::from(0).into(),
        type_parameters: vec![],
        visibility: Visibility::Public,
        is_contract_call: false,
        where_clause: vec![],
        is_trait_method_dummy: false,
        kind: ty::TyFunctionDeclKind::Default,
    };

    let selector_text = decl
        .to_selector_name(&handler, &engines)
        .expect("test failure");

    assert_eq!(selector_text, "foo()".to_string());

    let decl = ty::TyFunctionDecl {
        purity: Default::default(),
        name: Ident::new_with_override("bar".into(), Span::dummy()),
        implementing_type: None,
        implementing_for_typeid: None,
        body: ty::TyCodeBlock::default(),
        parameters: vec![
            ty::TyFunctionParameter {
                name: Ident::dummy(),
                is_reference: false,
                is_mutable: false,
                mutability_span: Span::dummy(),
                type_argument: engines
                    .te()
                    .insert(
                        &engines,
                        TypeInfo::StringArray(Length::new(5, Span::dummy())),
                        None,
                    )
                    .into(),
            },
            ty::TyFunctionParameter {
                name: Ident::new_no_span("baz".into()),
                is_reference: false,
                is_mutable: false,
                mutability_span: Span::dummy(),
                type_argument: TypeArgument {
                    type_id: engines.te().insert(
                        &engines,
                        TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo),
                        None,
                    ),
                    initial_type_id: engines.te().insert(
                        &engines,
                        TypeInfo::StringArray(Length::new(5, Span::dummy())),
                        None,
                    ),
                    span: Span::dummy(),
                    call_path_tree: None,
                },
            },
        ],
        span: Span::dummy(),
        call_path: CallPath::from(Ident::dummy()),
        attributes: Default::default(),
        return_type: TypeId::from(0).into(),
        type_parameters: vec![],
        visibility: Visibility::Public,
        is_contract_call: false,
        where_clause: vec![],
        is_trait_method_dummy: false,
        kind: ty::TyFunctionDeclKind::Default,
    };

    let selector_text = decl
        .to_selector_name(&handler, &engines)
        .expect("test failure");

    assert_eq!(selector_text, "bar(str[5],u32)".to_string());
}
