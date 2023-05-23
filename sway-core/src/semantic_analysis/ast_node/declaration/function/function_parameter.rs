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
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let type_engine = ctx.type_engine;
        let engines = ctx.engines();

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
            type_engine.insert(engines, TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );

        check!(
            type_argument
                .type_id
                .check_type_parameter_bounds(&ctx, &type_argument.span),
            return err(warnings, errors),
            warnings,
            errors
        );

        let mutability = ty::VariableMutability::new_from_ref_mut(is_reference, is_mutable);
        if mutability == ty::VariableMutability::Mutable {
            errors.push(CompileError::MutableParameterNotSupported {
                param_name: name.clone(),
                span: name.span(),
            });
            return err(warnings, errors);
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
        let engines = ctx.engines();

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
            type_engine.insert(engines, TypeInfo::ErrorRecovery),
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
