use crate::{
    error::{err, ok},
    language::{parsed::FunctionParameter, ty},
    semantic_analysis::TypeCheckContext,
    type_system::*,
    CompileResult,
};

use sway_error::error::CompileError;
use sway_types::{Ident, Spanned};

impl ty::TyFunctionParameter {
    pub(crate) fn type_check(
        mut ctx: TypeCheckContext,
        parameter: FunctionParameter,
        is_from_method: bool,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;

        let is_self = parameter.is_self();

        let FunctionParameter {
            name,
            is_reference,
            is_mutable,
            mutability_span,
            mut type_argument,
        } = parameter;

        let is_self_and_is_ascribed =
            is_self && !matches!(type_engine.get(type_argument.type_id), TypeInfo::SelfType);

        type_argument.type_id = check!(
            ctx.resolve_type_with_self(
                type_argument.type_id,
                &type_argument.span,
                EnforceTypeArguments::Yes,
                None
            ),
            type_engine.insert(decl_engine, TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );

        // If this parameter is `self` and if it is type ascribed, then check that the type
        // ascribed is `core::experimental::storage::StorageHandle`. Otherwise, emit an error. In
        // the future, we may want to allow more types such as `Self`.
        if is_self_and_is_ascribed {
            match type_engine.get(type_argument.type_id) {
                TypeInfo::Struct(decl_ref) => {
                    let struct_decl = decl_engine.get_struct(&decl_ref);
                    if !(struct_decl.call_path.prefixes
                        == vec![
                            Ident::new_no_span("core".into()),
                            Ident::new_no_span("experimental".into()),
                            Ident::new_no_span("storage".into()),
                        ]
                        && struct_decl.call_path.suffix
                            == Ident::new_no_span("StorageHandle".into()))
                    {
                        errors.push(CompileError::InvalidSelfParamterType {
                            r#type: ctx.engines().help_out(type_argument.type_id).to_string(),
                            span: name.span(),
                        });
                    }
                }
                _ => {
                    errors.push(CompileError::InvalidSelfParamterType {
                        r#type: ctx.engines().help_out(type_argument.type_id).to_string(),
                        span: name.span(),
                    });
                }
            }
        }

        if !is_from_method {
            let mutability = ty::VariableMutability::new_from_ref_mut(is_reference, is_mutable);
            if mutability == ty::VariableMutability::Mutable {
                errors.push(CompileError::MutableParameterNotSupported {
                    param_name: name.clone(),
                    span: name.span(),
                });
                return err(warnings, errors);
            }
        }

        let typed_parameter = ty::TyFunctionParameter {
            name,
            is_reference,
            is_mutable,
            mutability_span,
            type_argument,
        };

        insert_into_namespace(ctx, &typed_parameter);

        ok(typed_parameter, warnings, errors)
    }

    pub(crate) fn type_check_interface_parameter(
        mut ctx: TypeCheckContext,
        parameter: FunctionParameter,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;

        let FunctionParameter {
            name,
            is_reference,
            is_mutable,
            mutability_span,
            mut type_argument,
        } = parameter;

        type_argument.type_id = check!(
            ctx.resolve_type_with_self(
                type_argument.type_id,
                &type_argument.span,
                EnforceTypeArguments::Yes,
                None
            ),
            type_engine.insert(decl_engine, TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );

        let typed_parameter = ty::TyFunctionParameter {
            name,
            is_reference,
            is_mutable,
            mutability_span,
            type_argument,
        };

        ok(typed_parameter, warnings, errors)
    }
}

fn insert_into_namespace(ctx: TypeCheckContext, typed_parameter: &ty::TyFunctionParameter) {
    ctx.namespace.insert_symbol(
        typed_parameter.name.clone(),
        ty::TyDecl::VariableDecl(Box::new(ty::TyVariableDecl {
            name: typed_parameter.name.clone(),
            body: ty::TyExpression {
                expression: ty::TyExpressionVariant::FunctionParameter,
                return_type: typed_parameter.type_argument.type_id,
                span: typed_parameter.name.span(),
            },
            mutability: ty::VariableMutability::new_from_ref_mut(
                typed_parameter.is_reference,
                typed_parameter.is_mutable,
            ),
            return_type: typed_parameter.type_argument.type_id,
            type_ascription: typed_parameter.type_argument.clone(),
        })),
    );
}
