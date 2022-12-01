use crate::{
    error::{err, ok},
    language::{parsed::FunctionParameter, ty},
    semantic_analysis::TypeCheckContext,
    type_system::*,
    CompileResult,
};

use sway_error::error::CompileError;
use sway_types::Spanned;

impl ty::TyFunctionParameter {
    pub(crate) fn type_check(
        mut ctx: TypeCheckContext,
        parameter: FunctionParameter,
        is_from_method: bool,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let FunctionParameter {
            name,
            is_reference,
            is_mutable,
            mutability_span,
            type_info,
            type_span,
        } = parameter;

        let initial_type_id = ctx.type_engine.insert_type(type_info);

        let type_id = check!(
            ctx.resolve_type_with_self(
                initial_type_id,
                &type_span,
                EnforceTypeArguments::Yes,
                None
            ),
            ctx.type_engine.insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );

        if !is_from_method {
            let mutability = ty::VariableMutability::new_from_ref_mut(is_reference, is_mutable);
            if mutability == ty::VariableMutability::Mutable {
                errors.push(CompileError::MutableParameterNotSupported { param_name: name });
                return err(warnings, errors);
            }
        }

        let typed_parameter = ty::TyFunctionParameter {
            name,
            is_reference,
            is_mutable,
            mutability_span,
            type_id,
            initial_type_id,
            type_span,
        };

        insert_into_namespace(ctx, &typed_parameter);

        ok(typed_parameter, warnings, errors)
    }

    pub(crate) fn type_check_interface_parameter(
        ctx: TypeCheckContext,
        parameter: FunctionParameter,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let FunctionParameter {
            name,
            is_reference,
            is_mutable,
            mutability_span,
            type_info,
            type_span,
        } = parameter;

        let initial_type_id = ctx.type_engine.insert_type(type_info);

        let type_id = check!(
            ctx.namespace.resolve_type_with_self(
                ctx.type_engine,
                initial_type_id,
                ctx.type_engine.insert_type(TypeInfo::SelfType),
                &type_span,
                EnforceTypeArguments::Yes,
                None
            ),
            ctx.type_engine.insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );

        let typed_parameter = ty::TyFunctionParameter {
            name,
            is_reference,
            is_mutable,
            mutability_span,
            type_id,
            initial_type_id,
            type_span,
        };

        ok(typed_parameter, warnings, errors)
    }
}

fn insert_into_namespace(ctx: TypeCheckContext, typed_parameter: &ty::TyFunctionParameter) {
    ctx.namespace.insert_symbol(
        typed_parameter.name.clone(),
        ty::TyDeclaration::VariableDeclaration(Box::new(ty::TyVariableDeclaration {
            name: typed_parameter.name.clone(),
            body: ty::TyExpression {
                expression: ty::TyExpressionVariant::FunctionParameter,
                return_type: typed_parameter.type_id,
                span: typed_parameter.name.span(),
            },
            mutability: ty::VariableMutability::new_from_ref_mut(
                typed_parameter.is_reference,
                typed_parameter.is_mutable,
            ),
            return_type: typed_parameter.type_id,
            type_ascription: typed_parameter.type_id,
            type_ascription_span: Some(typed_parameter.type_span.clone()),
        })),
    );
}
