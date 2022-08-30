use crate::{
    declaration_engine::declaration_engine::DeclarationEngine,
    error::{err, ok},
    semantic_analysis::{
        convert_to_variable_immutability, IsConstant, TypeCheckContext, TypedExpression,
        TypedExpressionVariant, TypedVariableDeclaration, VariableMutability,
    },
    type_system::*,
    types::{CompileWrapper, ToCompileWrapper},
    CompileError, CompileResult, FunctionParameter, Ident, TypedDeclaration,
};

use sway_types::{span::Span, Spanned};

#[derive(Debug, Clone)]
pub struct TypedFunctionParameter {
    pub name: Ident,
    pub is_reference: bool,
    pub is_mutable: bool,
    pub type_id: TypeId,
    pub initial_type_id: TypeId,
    pub type_span: Span,
}

impl PartialEq for CompileWrapper<'_, TypedFunctionParameter> {
    fn eq(&self, other: &Self) -> bool {
        let CompileWrapper {
            inner: me,
            declaration_engine: de,
        } = self;
        let CompileWrapper { inner: them, .. } = other;
        me.name == them.name
            && look_up_type_id(me.type_id).wrap(de) == look_up_type_id(them.type_id).wrap(de)
            && me.is_mutable == them.is_mutable
    }
}

impl CopyTypes for TypedFunctionParameter {
    fn copy_types(&mut self, type_mapping: &TypeMapping, de: &DeclarationEngine) {
        self.type_id.update_type(type_mapping, de, &self.type_span);
    }
}

impl TypedFunctionParameter {
    pub fn is_self(&self) -> bool {
        self.name.as_str() == "self"
    }

    pub(crate) fn type_check(
        mut ctx: TypeCheckContext,
        parameter: FunctionParameter,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let type_id = check!(
            ctx.resolve_type_with_self(
                parameter.type_id,
                &parameter.type_span,
                EnforceTypeArguments::Yes,
                None
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );
        let mutability =
            convert_to_variable_immutability(parameter.is_reference, parameter.is_mutable);
        if mutability == VariableMutability::Mutable {
            errors.push(CompileError::MutableParameterNotSupported {
                param_name: parameter.name,
            });
            return err(warnings, errors);
        }
        ctx.namespace.insert_symbol(
            parameter.name.clone(),
            TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                name: parameter.name.clone(),
                body: TypedExpression {
                    expression: TypedExpressionVariant::FunctionParameter,
                    return_type: type_id,
                    is_constant: IsConstant::No,
                    span: parameter.name.span(),
                },
                mutability,
                type_ascription: type_id,
                type_ascription_span: None,
            }),
        );
        let parameter = TypedFunctionParameter {
            name: parameter.name,
            is_reference: parameter.is_reference,
            is_mutable: parameter.is_mutable,
            type_id,
            initial_type_id: parameter.type_id,
            type_span: parameter.type_span,
        };
        ok(parameter, warnings, errors)
    }
}
