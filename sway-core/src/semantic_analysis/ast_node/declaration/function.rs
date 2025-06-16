mod function_parameter;

use ast_elements::type_parameter::GenericTypeParameter;
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
    warning::{CompileWarning, Warning},
};
use symbol_collection_context::SymbolCollectionContext;

use crate::{
    decl_engine::{
        parsed_id::ParsedDeclId, DeclEngineInsert as _, DeclId, DeclRefFunction,
        ParsedDeclEngineGet as _,
    },
    language::{
        parsed::*,
        ty::{self, ConstGenericDecl, TyCodeBlock, TyConstGenericDecl, TyDecl, TyFunctionDecl},
        CallPath, CallPathType, Visibility,
    },
    semantic_analysis::*,
    type_system::*,
    Engines,
};
use sway_types::{style::is_snake_case, Spanned};

impl ty::TyFunctionDecl {
    pub(crate) fn collect(
        handler: &Handler,
        engines: &Engines,
        ctx: &mut SymbolCollectionContext,
        decl_id: &ParsedDeclId<FunctionDeclaration>,
    ) -> Result<(), ErrorEmitted> {
        let fn_decl = engines.pe().get_function(decl_id);
        let decl = Declaration::FunctionDeclaration(*decl_id);
        let _ = ctx.insert_parsed_symbol(handler, engines, fn_decl.name.clone(), decl.clone());

        // create a namespace for the function
        let _ = ctx.scoped(engines, fn_decl.span.clone(), Some(decl), |scoped_ctx| {
            // let const_generic_parameters = fn_decl
            //     .type_parameters
            //     .iter()
            //     .filter_map(|x| x.as_const_parameter())
            //     .filter_map(|x| x.id.as_ref());

            // for const_generic_parameter in const_generic_parameters {
            //     let const_generic_decl = engines.pe().get(const_generic_parameter);
            //     scoped_ctx.insert_parsed_symbol(
            //         handler,
            //         engines,
            //         const_generic_decl.name.clone(),
            //         Declaration::ConstGenericDeclaration(*const_generic_parameter),
            //     )?;
            // }

            TyCodeBlock::collect(handler, engines, scoped_ctx, &fn_decl.body)
        });
        Ok(())
    }

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
            .with_const_shadowing_mode(ConstShadowingMode::Sequential)
            .disallow_functions()
            .scoped(handler, Some(span.clone()), |ctx| {
                // Type check the type parameters.
                let new_type_parameters = GenericTypeParameter::type_check_type_params(
                    handler,
                    ctx.by_ref(),
                    type_parameters.clone(),
                    None,
                )?;

                // const generic parameters
                let const_generic_parameters = type_parameters
                    .iter()
                    .filter_map(|x| x.as_const_parameter())
                    .map(|x| &x.decl_ref);
                for p in const_generic_parameters {
                    ctx.insert_symbol(
                        handler,
                        p.name().clone(),
                        TyDecl::ConstGenericDecl(ConstGenericDecl { decl_id: *p.id() }),
                    )?;
                }

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
                *return_type.type_id_mut() = ctx
                    .resolve_type(
                        handler,
                        return_type.type_id(),
                        &return_type.span(),
                        EnforceTypeArguments::Yes,
                        None,
                    )
                    .unwrap_or_else(|err| type_engine.id_of_error_recovery(err));

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
                    body: <_>::default(),
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
                    is_type_check_finalized: false,
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
            .with_const_shadowing_mode(ConstShadowingMode::Sequential)
            .disallow_functions()
            .scoped(handler, Some(fn_decl.span.clone()), |ctx| {
                let FunctionDeclaration { body, .. } = fn_decl;

                let ty::TyFunctionDecl {
                    parameters,
                    return_type,
                    type_parameters,
                    ..
                } = ty_fn_decl;

                // Insert the previously type checked type parameters into the current namespace.
                // We insert all type parameter before the constraints because some constraints may depend on the parameters.
                for p in type_parameters.iter() {
                    p.insert_into_namespace_self(handler, ctx.by_ref())?;
                }
                for p in type_parameters.iter() {
                    p.insert_into_namespace_constraints(handler, ctx.by_ref())?;
                }

                // Insert the previously type checked function parameters into the current namespace.
                for p in parameters.iter() {
                    p.insert_into_namespace(handler, ctx.by_ref());
                }

                // type check the function body
                //
                // If there are no implicit block returns, then we do not want to type check them, so we
                // stifle the errors. If there _are_ implicit block returns, we want to type_check them.

                let mut ctx = ctx
                    .by_ref()
                    .with_help_text(
                        "Function body's return type does not match up with its return type annotation.",
                    )
                    .with_type_annotation(return_type.type_id())
                    .with_function_type_annotation(return_type.type_id());

                let body = ty::TyCodeBlock::type_check(handler, ctx.by_ref(), body, true)
                    .unwrap_or_else(|_err| ty::TyCodeBlock::default());

                ty_fn_decl.body = body;
                ty_fn_decl.is_type_check_finalized = true;

                return_type.type_id().check_type_parameter_bounds(
                    handler,
                    ctx.by_ref(),
                    &return_type.span(),
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
    use sway_types::{Ident, Span};

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
        is_type_check_finalized: true,
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
                    .insert_string_array_without_annotations(&engines, 5)
                    .into(),
            },
            ty::TyFunctionParameter {
                name: Ident::new_no_span("baz".into()),
                is_reference: false,
                is_mutable: false,
                mutability_span: Span::dummy(),
                type_argument: GenericArgument::Type(
                    ast_elements::type_argument::GenericTypeArgument {
                        type_id: engines.te().id_of_u32(),
                        initial_type_id: engines
                            .te()
                            .insert_string_array_without_annotations(&engines, 5),
                        span: Span::dummy(),
                        call_path_tree: None,
                    },
                ),
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
        is_type_check_finalized: true,
        kind: ty::TyFunctionDeclKind::Default,
    };

    let selector_text = decl
        .to_selector_name(&handler, &engines)
        .expect("test failure");

    assert_eq!(selector_text, "bar(str[5],u32)".to_string());
}
