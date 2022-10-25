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

        // Warn against non-snake case function names.
        if !is_snake_case(name.as_str()) {
            warnings.push(CompileWarning {
                span: name.span(),
                warning_content: Warning::NonSnakeCaseFunctionName { name: name.clone() },
            })
        }

        // create a namespace for the function
        let mut fn_namespace = ctx.namespace.clone();
        let mut fn_ctx = ctx.by_ref().scoped(&mut fn_namespace).with_purity(purity);

        // type check the type parameters
        // insert them into the namespace
        let mut new_type_parameters = vec![];
        for type_parameter in type_parameters.into_iter() {
            if !type_parameter.trait_constraints.is_empty() {
                errors.push(CompileError::WhereClauseNotYetSupported {
                    span: type_parameter.trait_constraints_span,
                });
                return err(warnings, errors);
            }
            new_type_parameters.push(check!(
                TypeParameter::type_check(fn_ctx.by_ref(), type_parameter),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

        // type check the function parameters
        // insert them into the namespace
        let mut new_parameters = vec![];
        for parameter in parameters.into_iter() {
            new_parameters.push(check!(
                ty::TyFunctionParameter::type_check(fn_ctx.by_ref(), parameter, is_method),
                continue,
                warnings,
                errors
            ));
        }

        // type check the return type
        let initial_return_type = insert_type(return_type);
        let return_type = check!(
            fn_ctx.resolve_type_with_self(
                initial_return_type,
                &return_type_span,
                EnforceTypeArguments::Yes,
                None
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );

        // type check the function body
        //
        // If there are no implicit block returns, then we do not want to type check them, so we
        // stifle the errors. If there _are_ implicit block returns, we want to type_check them.
        let (body, _implicit_block_return) = {
            let fn_ctx = fn_ctx
                .by_ref()
                .with_purity(purity)
                .with_help_text("Function body's return type does not match up with its return type annotation.")
                .with_type_annotation(return_type);
            check!(
                ty::TyCodeBlock::type_check(fn_ctx, body),
                (
                    ty::TyCodeBlock { contents: vec![] },
                    insert_type(TypeInfo::ErrorRecovery)
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

        // unify the types of the return statements with the function return type
        for stmt in return_statements {
            append!(
                fn_ctx
                    .by_ref()
                    .with_type_annotation(return_type)
                    .with_help_text(
                        "Return statement must return the declared function return type."
                    )
                    .unify_with_self(stmt.return_type, &stmt.span),
                warnings,
                errors
            );
        }

        let (visibility, is_contract_call) = if is_method {
            (Visibility::Public, false)
        } else {
            (visibility, fn_ctx.mode() == Mode::ImplAbiFn)
        };

        let function_decl = ty::TyFunctionDeclaration {
            name,
            body,
            parameters: new_parameters,
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

        // Retrieve the implemented traits for the type of the return type and
        // insert them in the broader namespace.
        ctx.namespace.implemented_traits.extend(
            fn_ctx
                .namespace
                .implemented_traits
                .filter_by_type(function_decl.return_type),
        );

        ok(function_decl, warnings, errors)
    }
}

#[test]
fn test_function_selector_behavior() {
    use crate::language::Visibility;
    use sway_types::{integer_bits::IntegerBits, Ident, Span};
    let decl = ty::TyFunctionDeclaration {
        purity: Default::default(),
        name: Ident::new_no_span("foo"),
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

    let selector_text = match decl.to_selector_name().value {
        Some(value) => value,
        _ => panic!("test failure"),
    };

    assert_eq!(selector_text, "foo()".to_string());

    let decl = ty::TyFunctionDeclaration {
        purity: Default::default(),
        name: Ident::new_with_override("bar", Span::dummy()),
        body: ty::TyCodeBlock { contents: vec![] },
        parameters: vec![
            ty::TyFunctionParameter {
                name: Ident::new_no_span("foo"),
                is_reference: false,
                is_mutable: false,
                mutability_span: Span::dummy(),
                type_id: crate::type_system::insert_type(TypeInfo::Str(5)),
                initial_type_id: crate::type_system::insert_type(TypeInfo::Str(5)),
                type_span: Span::dummy(),
            },
            ty::TyFunctionParameter {
                name: Ident::new_no_span("baz"),
                is_reference: false,
                is_mutable: false,
                mutability_span: Span::dummy(),
                type_id: insert_type(TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo)),
                initial_type_id: crate::type_system::insert_type(TypeInfo::Str(5)),
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

    let selector_text = match decl.to_selector_name().value {
        Some(value) => value,
        _ => panic!("test failure"),
    };

    assert_eq!(selector_text, "bar(str[5],u32)".to_string());
}
