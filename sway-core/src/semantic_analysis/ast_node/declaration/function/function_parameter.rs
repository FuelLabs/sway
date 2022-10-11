use crate::{
    error::{err, ok},
    language::{parsed::FunctionParameter, ty},
    semantic_analysis::{
        convert_to_variable_immutability, IsConstant, TyVariableDeclaration, TypeCheckContext,
        VariableMutability,
    },
    type_system::*,
    CompileResult, Ident, Namespace, TyDeclaration,
};

use sway_error::error::CompileError;
use sway_types::{span::Span, Spanned};

#[derive(Debug, Clone, Eq)]
pub struct TyFunctionParameter {
    pub name: Ident,
    pub is_reference: bool,
    pub is_mutable: bool,
    pub mutability_span: Span,
    pub type_id: TypeId,
    pub initial_type_id: TypeId,
    pub type_span: Span,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TyFunctionParameter {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && look_up_type_id(self.type_id) == look_up_type_id(other.type_id)
            && self.is_mutable == other.is_mutable
    }
}

impl CopyTypes for TyFunctionParameter {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.type_id.copy_types(type_mapping);
    }
}

impl TyFunctionParameter {
    pub fn is_self(&self) -> bool {
        self.name.as_str() == "self"
    }

    pub(crate) fn type_check(
        mut ctx: TypeCheckContext,
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

        let initial_type_id = insert_type(type_info);

        let type_id = check!(
            ctx.resolve_type_with_self(
                initial_type_id,
                &type_span,
                EnforceTypeArguments::Yes,
                None
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );

        let mutability = convert_to_variable_immutability(is_reference, is_mutable);
        if mutability == VariableMutability::Mutable {
            errors.push(CompileError::MutableParameterNotSupported { param_name: name });
            return err(warnings, errors);
        }

        let typed_parameter = TyFunctionParameter {
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

    pub(crate) fn type_check_method_parameter(
        mut ctx: TypeCheckContext,
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

        let initial_type_id = insert_type(type_info);

        let type_id = check!(
            ctx.resolve_type_with_self(
                initial_type_id,
                &type_span,
                EnforceTypeArguments::Yes,
                None
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );

        let typed_parameter = TyFunctionParameter {
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
        namespace: &mut Namespace,
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

        let initial_type_id = insert_type(type_info);

        let type_id = check!(
            namespace.resolve_type_with_self(
                initial_type_id,
                insert_type(TypeInfo::SelfType),
                &type_span,
                EnforceTypeArguments::Yes,
                None
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );

        let typed_parameter = TyFunctionParameter {
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

fn insert_into_namespace(ctx: TypeCheckContext, typed_parameter: &TyFunctionParameter) {
    ctx.namespace.insert_symbol(
        typed_parameter.name.clone(),
        TyDeclaration::VariableDeclaration(Box::new(TyVariableDeclaration {
            name: typed_parameter.name.clone(),
            body: ty::TyExpression {
                expression: ty::TyExpressionVariant::FunctionParameter,
                return_type: typed_parameter.type_id,
                is_constant: IsConstant::No,
                span: typed_parameter.name.span(),
            },
            mutability: convert_to_variable_immutability(
                typed_parameter.is_reference,
                typed_parameter.is_mutable,
            ),
            type_ascription: typed_parameter.type_id,
            type_ascription_span: Some(typed_parameter.type_span.clone()),
        })),
    );
}
