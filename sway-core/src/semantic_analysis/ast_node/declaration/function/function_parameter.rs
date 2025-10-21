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
        ctx: TypeCheckContext,
        parameter: FunctionParameter,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();

        let FunctionParameter {
            name,
            is_reference,
            is_mutable,
            mutability_span,
            mut type_argument,
        } = parameter;

        type_argument.type_id = ctx
            .resolve_type(
                handler,
                type_argument.type_id,
                &type_argument.span(),
                EnforceTypeArguments::Yes,
                None,
            )
            .unwrap_or_else(|err| type_engine.id_of_error_recovery(err));

        type_argument.type_id.check_type_parameter_bounds(
            handler,
            ctx,
            &type_argument.span(),
            None,
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

        Ok(typed_parameter)
    }

    pub(crate) fn type_check_interface_parameter(
        handler: &Handler,
        ctx: TypeCheckContext,
        parameter: &FunctionParameter,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();

        let FunctionParameter {
            name,
            is_reference,
            is_mutable,
            mutability_span,
            type_argument,
        } = parameter;

        let mut new_type_argument = type_argument.clone();
        new_type_argument.type_id = ctx
            .resolve_type(
                handler,
                type_argument.type_id,
                &type_argument.span,
                EnforceTypeArguments::Yes,
                None,
            )
            .unwrap_or_else(|err| type_engine.id_of_error_recovery(err));

        let typed_parameter = ty::TyFunctionParameter {
            name: name.clone(),
            is_reference: *is_reference,
            is_mutable: *is_mutable,
            mutability_span: mutability_span.clone(),
            type_argument: new_type_argument,
        };

        Ok(typed_parameter)
    }

    pub fn insert_into_namespace(&self, handler: &Handler, mut ctx: TypeCheckContext) {
        let _ = ctx.insert_symbol(
            handler,
            self.name.clone(),
            ty::TyDecl::VariableDecl(Box::new(ty::TyVariableDecl {
                name: self.name.clone(),
                body: ty::TyExpression {
                    expression: ty::TyExpressionVariant::FunctionParameter,
                    return_type: self.type_argument.type_id,
                    span: self.name.span(),
                },
                mutability: ty::VariableMutability::new_from_ref_mut(
                    self.is_reference,
                    self.is_mutable,
                ),
                return_type: self.type_argument.type_id,
                type_ascription: self.type_argument.clone(),
            })),
        );
    }
}
