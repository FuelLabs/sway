mod function_parameter;

pub use function_parameter::*;
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
    warning::{CompileWarning, Warning},
};

use crate::{
    language::{
        parsed::*,
        ty::{self, TyCodeBlock},
        Visibility,
    },
    semantic_analysis::*,
    type_system::*,
};
use sway_types::{style::is_snake_case, Spanned};

impl ty::TyFunctionDecl {
    pub fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        fn_decl: FunctionDeclaration,
        is_method: bool,
        is_in_impl_self: bool,
    ) -> Result<Self, ErrorEmitted> {
        let mut ty_fn_decl = Self::type_check_signature(
            handler,
            ctx.by_ref(),
            fn_decl.clone(),
            is_method,
            is_in_impl_self,
        )?;
        Self::type_check_body(handler, ctx, fn_decl, &mut ty_fn_decl)
    }

    pub fn type_check_signature(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        fn_decl: FunctionDeclaration,
        is_method: bool,
        is_in_impl_self: bool,
    ) -> Result<Self, ErrorEmitted> {
        let FunctionDeclaration {
            name,
            body: _,
            parameters,
            span,
            attributes,
            mut return_type,
            type_parameters,
            visibility,
            purity,
            where_clause,
        } = fn_decl;

        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        // If functions aren't allowed in this location, return an error.
        if ctx.functions_disallowed() {
            return Err(handler.emit_err(CompileError::Unimplemented(
                "Nested function definitions are not allowed at this time.",
                span,
            )));
        }

        // Warn against non-snake case function names.
        if !is_snake_case(name.as_str()) {
            handler.emit_warn(CompileWarning {
                span: name.span(),
                warning_content: Warning::NonSnakeCaseFunctionName { name: name.clone() },
            })
        }

        // create a namespace for the function
        let mut fn_namespace = ctx.namespace.clone();
        let mut ctx = ctx
            .by_ref()
            .scoped(&mut fn_namespace)
            .with_purity(purity)
            .with_const_shadowing_mode(ConstShadowingMode::Sequential)
            .disallow_functions();

        // Type check the type parameters. This will also insert them into the
        // current namespace.
        let new_type_parameters =
            TypeParameter::type_check_type_params(handler, ctx.by_ref(), type_parameters, None)?;

        // Insert them into the current namespace.
        for p in &new_type_parameters {
            p.insert_into_namespace(handler, ctx.by_ref())?;
        }

        // type check the function parameters, which will also insert them into the namespace
        let mut new_parameters = vec![];
        handler.scope(|handler| {
            for parameter in parameters.into_iter() {
                new_parameters.push({
                    let param =
                        match ty::TyFunctionParameter::type_check(handler, ctx.by_ref(), parameter)
                        {
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
            .unwrap_or_else(|err| type_engine.insert(engines, TypeInfo::ErrorRecovery(err)));

        let (visibility, is_contract_call) = if is_method {
            if is_in_impl_self {
                (visibility, false)
            } else {
                (Visibility::Public, false)
            }
        } else {
            (visibility, matches!(ctx.abi_mode(), AbiMode::ImplAbiFn(..)))
        };

        let function_decl = ty::TyFunctionDecl {
            name,
            body: TyCodeBlock::default(),
            parameters: new_parameters,
            implementing_type: None,
            span,
            attributes,
            return_type,
            type_parameters: new_type_parameters,
            visibility,
            is_contract_call,
            purity,
            where_clause,
        };

        Ok(function_decl)
    }

    pub fn type_check_body(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        fn_decl: FunctionDeclaration,
        ty_fn_decl: &mut Self,
    ) -> Result<Self, ErrorEmitted> {
        let FunctionDeclaration { body, .. } = fn_decl;

        let ty::TyFunctionDecl {
            parameters,
            purity,
            return_type,
            type_parameters,
            ..
        } = ty_fn_decl;

        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        // create a namespace for the function
        let mut fn_namespace = ctx.namespace.clone();
        let mut ctx = ctx
            .by_ref()
            .scoped(&mut fn_namespace)
            .with_purity(*purity)
            .with_const_shadowing_mode(ConstShadowingMode::Sequential)
            .disallow_functions();

        // Insert the previously type checked type parameters into the current namespace.
        for p in type_parameters {
            p.insert_into_namespace(handler, ctx.by_ref())?;
        }

        // Insert the previously type checked function parameters into the namespace.
        for p in parameters {
            p.insert_into_namespace(handler, ctx.by_ref());
        }

        // type check the function body
        //
        // If there are no implicit block returns, then we do not want to type check them, so we
        // stifle the errors. If there _are_ implicit block returns, we want to type_check them.
        let (body, _implicit_block_return) = {
            let ctx = ctx
                .by_ref()
                .with_purity(*purity)
                .with_help_text("Function body's return type does not match up with its return type annotation.")
                .with_type_annotation(return_type.type_id);
            ty::TyCodeBlock::type_check(handler, ctx, body).unwrap_or_else(|err| {
                (
                    ty::TyCodeBlock { contents: vec![] },
                    type_engine.insert(engines, TypeInfo::ErrorRecovery(err)),
                )
            })
        };

        // gather the return statements
        let return_statements: Vec<&ty::TyExpression> = body
            .contents
            .iter()
            .flat_map(|node| node.gather_return_statements())
            .collect();

        unify_return_statements(
            handler,
            ctx.by_ref(),
            &return_statements,
            return_type.type_id,
        )?;

        return_type.type_id.check_type_parameter_bounds(
            handler,
            &ctx,
            &return_type.span,
            vec![],
        )?;

        ty_fn_decl.body = body;
        Ok(ty_fn_decl.clone())
    }
}

/// Unifies the types of the return statements and the return type of the
/// function declaration.
fn unify_return_statements(
    handler: &Handler,
    ctx: TypeCheckContext,
    return_statements: &[&ty::TyExpression],
    return_type: TypeId,
) -> Result<(), ErrorEmitted> {
    let type_engine = ctx.engines.te();

    handler.scope(|handler| {
        for stmt in return_statements.iter() {
            type_engine.unify(
                handler,
                ctx.engines(),
                stmt.return_type,
                return_type,
                &stmt.span,
                "Return statement must return the declared function return type.",
                None,
            );
        }
        Ok(())
    })
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
        name: Ident::new_no_span("foo".into()),
        implementing_type: None,
        body: ty::TyCodeBlock { contents: vec![] },
        parameters: vec![],
        span: Span::dummy(),
        attributes: Default::default(),
        return_type: TypeId::from(0).into(),
        type_parameters: vec![],
        visibility: Visibility::Public,
        is_contract_call: false,
        where_clause: vec![],
    };

    let selector_text = decl
        .to_selector_name(&handler, &engines)
        .expect("test failure");

    assert_eq!(selector_text, "foo()".to_string());

    let decl = ty::TyFunctionDecl {
        purity: Default::default(),
        name: Ident::new_with_override("bar".into(), Span::dummy()),
        implementing_type: None,
        body: ty::TyCodeBlock { contents: vec![] },
        parameters: vec![
            ty::TyFunctionParameter {
                name: Ident::new_no_span("foo".into()),
                is_reference: false,
                is_mutable: false,
                mutability_span: Span::dummy(),
                type_argument: engines
                    .te()
                    .insert(&engines, TypeInfo::Str(Length::new(5, Span::dummy())))
                    .into(),
            },
            ty::TyFunctionParameter {
                name: Ident::new_no_span("baz".into()),
                is_reference: false,
                is_mutable: false,
                mutability_span: Span::dummy(),
                type_argument: TypeArgument {
                    type_id: engines
                        .te()
                        .insert(&engines, TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo)),
                    initial_type_id: engines
                        .te()
                        .insert(&engines, TypeInfo::Str(Length::new(5, Span::dummy()))),
                    span: Span::dummy(),
                    call_path_tree: None,
                },
            },
        ],
        span: Span::dummy(),
        attributes: Default::default(),
        return_type: TypeId::from(0).into(),
        type_parameters: vec![],
        visibility: Visibility::Public,
        is_contract_call: false,
        where_clause: vec![],
    };

    let selector_text = decl
        .to_selector_name(&handler, &engines)
        .expect("test failure");

    assert_eq!(selector_text, "bar(str[5],u32)".to_string());
}
