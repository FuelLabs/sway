use super::code_block::desugar_code_block;
use super::expression::desugar_expression;
use crate::error::{err, ok, CompileResult};
use crate::{
    AbiDeclaration, ConstantDeclaration, Declaration, EnumDeclaration, FunctionDeclaration,
    ImplSelf, ImplTrait, Reassignment, StructDeclaration, TraitDeclaration, VariableDeclaration,
};

pub fn desugar_declaration<'sc>(decl: Declaration<'sc>) -> CompileResult<'sc, Declaration<'sc>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let decl = match decl {
        Declaration::Reassignment(reassignment) => Declaration::Reassignment(check!(
            desugar_reassignment(reassignment),
            return err(warnings, errors),
            warnings,
            errors
        )),
        Declaration::StructDeclaration(struct_decl) => Declaration::StructDeclaration(check!(
            desugar_struct_decl(struct_decl),
            return err(warnings, errors),
            warnings,
            errors
        )),
        Declaration::TraitDeclaration(trait_decl) => Declaration::TraitDeclaration(check!(
            desugar_trait_decl(trait_decl),
            return err(warnings, errors),
            warnings,
            errors
        )),
        Declaration::ImplTrait(impl_trait) => Declaration::ImplTrait(check!(
            desugar_impl_trait(impl_trait),
            return err(warnings, errors),
            warnings,
            errors
        )),
        Declaration::ImplSelf(impl_self) => Declaration::ImplSelf(check!(
            desugar_impl_self(impl_self),
            return err(warnings, errors),
            warnings,
            errors
        )),
        Declaration::EnumDeclaration(enum_decl) => Declaration::EnumDeclaration(check!(
            desugar_enum_decl(enum_decl),
            return err(warnings, errors),
            warnings,
            errors
        )),
        Declaration::ConstantDeclaration(const_decl) => Declaration::ConstantDeclaration(check!(
            desugar_const_decl(const_decl),
            return err(warnings, errors),
            warnings,
            errors
        )),
        Declaration::AbiDeclaration(abi_decl) => Declaration::AbiDeclaration(check!(
            desugar_abi_decl(abi_decl),
            return err(warnings, errors),
            warnings,
            errors
        )),
        Declaration::FunctionDeclaration(function_decl) => {
            Declaration::FunctionDeclaration(check!(
                desugar_function_declaration(function_decl),
                return err(warnings, errors),
                warnings,
                errors
            ))
        }
        Declaration::VariableDeclaration(var_decl) => Declaration::VariableDeclaration(check!(
            desugar_variable_declaration(var_decl),
            return err(warnings, errors),
            warnings,
            errors
        )),
    };
    ok(decl, warnings, errors)
}

fn desugar_function_declaration<'sc>(
    function_decl: FunctionDeclaration<'sc>,
) -> CompileResult<'sc, FunctionDeclaration<'sc>> {
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
        type_parameters: function_decl.type_parameters,
    };
    ok(function_decl, warnings, errors)
}

fn desugar_variable_declaration<'sc>(
    var_decl: VariableDeclaration<'sc>,
) -> CompileResult<'sc, VariableDeclaration<'sc>> {
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
        ),
    };
    ok(var_decl, warnings, errors)
}

fn desugar_abi_decl<'sc>(abi_decl: AbiDeclaration<'sc>) -> CompileResult<'sc, AbiDeclaration<'sc>> {
    unimplemented!()
}

fn desugar_const_decl<'sc>(
    const_decl: ConstantDeclaration<'sc>,
) -> CompileResult<'sc, ConstantDeclaration<'sc>> {
    unimplemented!()
}

fn desugar_enum_decl<'sc>(
    enum_decl: EnumDeclaration<'sc>,
) -> CompileResult<'sc, EnumDeclaration<'sc>> {
    unimplemented!()
}

fn desugar_impl_self<'sc>(impl_self: ImplSelf<'sc>) -> CompileResult<'sc, ImplSelf<'sc>> {
    unimplemented!()
}

fn desugar_impl_trait<'sc>(impl_trait: ImplTrait<'sc>) -> CompileResult<'sc, ImplTrait<'sc>> {
    unimplemented!()
}

fn desugar_trait_decl<'sc>(
    trait_decl: TraitDeclaration<'sc>,
) -> CompileResult<'sc, TraitDeclaration<'sc>> {
    unimplemented!()
}

fn desugar_struct_decl<'sc>(
    struct_decl: StructDeclaration<'sc>,
) -> CompileResult<'sc, StructDeclaration<'sc>> {
    unimplemented!()
}

fn desugar_reassignment<'sc>(
    reassignment: Reassignment<'sc>,
) -> CompileResult<'sc, Reassignment<'sc>> {
    unimplemented!()
}
