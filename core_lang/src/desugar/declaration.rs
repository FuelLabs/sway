use crate::{Declaration, FunctionDeclaration, VariableDeclaration};
use crate::error::{err, ok, CompileResult};
use super::code_block::desugar_code_block;
use super::expression::desugar_expression;

pub fn desugar_declaration<'sc>(decl: Declaration<'sc>) -> CompileResult<'sc, Declaration<'sc>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    match decl {
        Declaration::FunctionDeclaration(function_decl) => {
            let decl = Declaration::FunctionDeclaration(check!(
                desugar_function_declaration(function_decl),
                return err(warnings, errors),
                warnings,
                errors
            ));
            ok(decl, warnings, errors)
        },
        Declaration::VariableDeclaration(var_decl) => {
            let decl = Declaration::VariableDeclaration(check!(
                desugar_variable_declaration(var_decl),
                return err(warnings, errors),
                warnings,
                errors
            ));
            ok(decl, warnings, errors)
        }
        decl => unimplemented!("{:?}", decl)
    }
}

fn desugar_function_declaration<'sc>(function_decl: FunctionDeclaration<'sc>) -> CompileResult<'sc, FunctionDeclaration<'sc>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let function_decl = FunctionDeclaration {
        name: function_decl.name,
        visibility: function_decl.visibility,
        body: check!(
            desugar_code_block(function_decl.body),
            return err(warnings, errors),
            warnings,
            errors
        ),
        parameters: function_decl.parameters,
        return_type: function_decl.return_type,
        return_type_span: function_decl.return_type_span,
        span: function_decl.span,
        type_parameters: function_decl.type_parameters
    };
    ok(function_decl, warnings, errors)
}

fn desugar_variable_declaration<'sc>(var_decl: VariableDeclaration<'sc>) -> CompileResult<'sc, VariableDeclaration<'sc>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let var_decl = VariableDeclaration {
        name: var_decl.name,
        type_ascription: var_decl.type_ascription,
        type_ascription_span: var_decl.type_ascription_span,
        is_mutable: var_decl.is_mutable,
        body: check!(
            desugar_expression(var_decl.body),
            return err(warnings, errors),
            warnings,
            errors
        )
    };
    ok(var_decl, warnings, errors)
}