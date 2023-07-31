use crate::{
    language::{parsed::FunctionParameter, ty},
    semantic_analysis::TypeCheckContext,
    type_system::*,
};

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::Spanned;

impl ty::TyFunctionParameter {
    pub(crate) fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        parameter: FunctionParameter,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let FunctionParameter {
            name,
            is_reference,
            is_mutable,
            mutability_span,
            mut type_argument,
        } = parameter;

        type_argument.type_id = ctx
            .resolve_type_with_self(
                handler,
                type_argument.type_id,
                &type_argument.span,
                EnforceTypeArguments::Yes,
                None,
            )
            .unwrap_or_else(|_| type_engine.insert(engines, TypeInfo::ErrorRecovery));

        type_argument.type_id.check_type_parameter_bounds(
            handler,
            &ctx,
            &type_argument.span,
            vec![],
        )?;

        let mutability = ty::VariableMutability::new_from_ref_mut(is_reference, is_mutable);
        if mutability == ty::VariableMutability::Mutable {
            return Err(
                handler.emit_err(CompileError::MutableParameterNotSupported {
                    param_name: name.clone(),
                    span: name.span(),
                }),
            );
        }

        let typed_parameter = ty::TyFunctionParameter {
            name,
            is_reference,
            is_mutable,
            mutability_span,
            type_argument,
        };

        insert_into_namespace(handler, ctx, &typed_parameter);

        Ok(typed_parameter)
    }

    pub(crate) fn type_check_interface_parameter(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        parameter: FunctionParameter,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let FunctionParameter {
            name,
            is_reference,
            is_mutable,
            mutability_span,
            mut type_argument,
        } = parameter;

        type_argument.type_id = ctx
            .resolve_type_with_self(
                handler,
                type_argument.type_id,
                &type_argument.span,
                EnforceTypeArguments::Yes,
                None,
            )
            .unwrap_or_else(|_| type_engine.insert(engines, TypeInfo::ErrorRecovery));

        let typed_parameter = ty::TyFunctionParameter {
            name,
            is_reference,
            is_mutable,
            mutability_span,
            type_argument,
        };

        Ok(typed_parameter)
    }
}

fn insert_into_namespace(
    handler: &Handler,
    ctx: TypeCheckContext,
    typed_parameter: &ty::TyFunctionParameter,
) {
    let const_shadowing_mode = ctx.const_shadowing_mode();
    let _ = ctx.namespace.insert_symbol(
        handler,
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
        const_shadowing_mode,
    );
}
