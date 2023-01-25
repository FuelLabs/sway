mod function_parameter;

pub use function_parameter::*;
use sway_error::{
    error::CompileError,
    warning::{CompileWarning, Warning},
};

use crate::{
    error::*,
    language::{parsed::*, ty, Visibility},
    semantic_analysis::*,
    type_system::*,
};
use sway_types::{style::is_snake_case, Spanned};

impl ty::TyFunctionDeclaration {
    pub fn type_check(
        mut ctx: TypeCheckContext,
        fn_decl: FunctionDeclaration,
        is_method: bool,
        is_in_impl_self: bool,
    ) -> CompileResult<Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let FunctionDeclaration {
            name,
            body,
            parameters,
            span,
            attributes,
            return_type,
            type_parameters,
            return_type_span,
            visibility,
            purity,
        } = fn_decl;

        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;

        // If functions aren't allowed in this location, return an error.
        if ctx.functions_disallowed() {
            errors.push(CompileError::Unimplemented(
                "Nested function definitions are not allowed at this time.",
                span,
            ));
            return err(warnings, errors);
        }

        // Warn against non-snake case function names.
        if !is_snake_case(name.as_str()) {
            warnings.push(CompileWarning {
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
            .disallow_functions();

        // type check the type parameters, which will also insert them into the namespace
        let mut new_type_parameters = vec![];
        for type_parameter in type_parameters.into_iter() {
            new_type_parameters.push(check!(
                TypeParameter::type_check(ctx.by_ref(), type_parameter),
                continue,
                warnings,
                errors
            ));
        }
        if !errors.is_empty() {
            return err(warnings, errors);
        }

        // type check the function parameters, which will also insert them into the namespace
        let mut new_parameters = vec![];
        for parameter in parameters.into_iter() {
            new_parameters.push(check!(
                ty::TyFunctionParameter::type_check(ctx.by_ref(), parameter, is_method),
                continue,
                warnings,
                errors
            ));
        }
        if !errors.is_empty() {
            return err(warnings, errors);
        }

        // type check the return type
        let initial_return_type = type_engine.insert(decl_engine, return_type);
        let return_type = check!(
            ctx.resolve_type_with_self(
                initial_return_type,
                &return_type_span,
                EnforceTypeArguments::Yes,
                None
            ),
            type_engine.insert(decl_engine, TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );

        // type check the function body
        //
        // If there are no implicit block returns, then we do not want to type check them, so we
        // stifle the errors. If there _are_ implicit block returns, we want to type_check them.
        let (body, _implicit_block_return) = {
            let fn_ctx = ctx
                .by_ref()
                .with_purity(purity)
                .with_help_text("Function body's return type does not match up with its return type annotation.")
                .with_type_annotation(return_type);
            check!(
                ty::TyCodeBlock::type_check(fn_ctx, body),
                (
                    ty::TyCodeBlock { contents: vec![] },
                    type_engine.insert(decl_engine, TypeInfo::ErrorRecovery)
                ),
                warnings,
                errors
            )
        };

        // gather the return statements
        let return_statements: Vec<&ty::TyExpression> = body
            .contents
            .iter()
            .flat_map(|node| node.gather_return_statements())
            .collect();

        check!(
            unify_return_statements(ctx.by_ref(), &return_statements, return_type),
            return err(warnings, errors),
            warnings,
            errors
        );

        let (visibility, is_contract_call) = if is_method {
            if is_in_impl_self {
                (visibility, false)
            } else {
                (Visibility::Public, false)
            }
        } else {
            (visibility, ctx.mode() == Mode::ImplAbiFn)
        };

        let function_decl = ty::TyFunctionDeclaration {
            name,
            body,
            parameters: new_parameters,
            implementing_type: None,
            span,
            attributes,
            return_type,
            initial_return_type,
            type_parameters: new_type_parameters,
            return_type_span,
            visibility,
            is_contract_call,
            purity,
        };

        ok(function_decl, warnings, errors)
    }
}

/// Unifies the types of the return statements and the return type of the
/// function declaration.
fn unify_return_statements(
    ctx: TypeCheckContext,
    return_statements: &[&ty::TyExpression],
    return_type: TypeId,
) -> CompileResult<()> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let type_engine = ctx.type_engine;
    let decl_engine = ctx.decl_engine;

    for stmt in return_statements.iter() {
        check!(
            CompileResult::from(type_engine.unify_with_self(
                decl_engine,
                stmt.return_type,
                return_type,
                ctx.self_type(),
                &stmt.span,
                "Return statement must return the declared function return type.",
                None,
            )),
            continue,
            warnings,
            errors
        );
    }

    if errors.is_empty() {
        ok((), warnings, errors)
    } else {
        err(warnings, errors)
    }
}

#[test]
fn test_function_selector_behavior() {
    use crate::{decl_engine::DeclEngine, language::Visibility};
    use sway_types::{integer_bits::IntegerBits, Ident, Span};

    let type_engine = TypeEngine::default();
    let decl_engine = DeclEngine::default();

    let decl = ty::TyFunctionDeclaration {
        purity: Default::default(),
        name: Ident::new_no_span("foo"),
        implementing_type: None,
        body: ty::TyCodeBlock { contents: vec![] },
        parameters: vec![],
        span: Span::dummy(),
        attributes: Default::default(),
        return_type: 0.into(),
        initial_return_type: 0.into(),
        type_parameters: vec![],
        return_type_span: Span::dummy(),
        visibility: Visibility::Public,
        is_contract_call: false,
    };

    let selector_text = match decl.to_selector_name(&type_engine).value {
        Some(value) => value,
        _ => panic!("test failure"),
    };

    assert_eq!(selector_text, "foo()".to_string());

    let decl = ty::TyFunctionDeclaration {
        purity: Default::default(),
        name: Ident::new_with_override("bar", Span::dummy()),
        implementing_type: None,
        body: ty::TyCodeBlock { contents: vec![] },
        parameters: vec![
            ty::TyFunctionParameter {
                name: Ident::new_no_span("foo"),
                is_reference: false,
                is_mutable: false,
                mutability_span: Span::dummy(),
                type_id: type_engine
                    .insert(&decl_engine, TypeInfo::Str(Length::new(5, Span::dummy()))),
                initial_type_id: type_engine
                    .insert(&decl_engine, TypeInfo::Str(Length::new(5, Span::dummy()))),
                type_span: Span::dummy(),
            },
            ty::TyFunctionParameter {
                name: Ident::new_no_span("baz"),
                is_reference: false,
                is_mutable: false,
                mutability_span: Span::dummy(),
                type_id: type_engine.insert(
                    &decl_engine,
                    TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo),
                ),
                initial_type_id: type_engine
                    .insert(&decl_engine, TypeInfo::Str(Length::new(5, Span::dummy()))),
                type_span: Span::dummy(),
            },
        ],
        span: Span::dummy(),
        attributes: Default::default(),
        return_type: 0.into(),
        initial_return_type: 0.into(),
        type_parameters: vec![],
        return_type_span: Span::dummy(),
        visibility: Visibility::Public,
        is_contract_call: false,
    };

    let selector_text = match decl.to_selector_name(&type_engine).value {
        Some(value) => value,
        _ => panic!("test failure"),
    };

    assert_eq!(selector_text, "bar(str[5],u32)".to_string());
}
