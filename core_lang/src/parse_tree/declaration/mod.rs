mod abi_declaration;
mod constant_declaration;
mod enum_declaration;
pub mod function_declaration;
mod impl_trait;
mod reassignment;
mod struct_declaration;
mod trait_declaration;
mod type_parameter;
mod variable_declaration;

pub(crate) use abi_declaration::*;
pub(crate) use constant_declaration::*;
pub(crate) use enum_declaration::*;
pub use function_declaration::*;
pub(crate) use impl_trait::*;
pub(crate) use reassignment::*;
pub use struct_declaration::*;
pub(crate) use trait_declaration::*;
pub(crate) use type_parameter::*;
pub use variable_declaration::*;

use crate::build_config::BuildConfig;
use crate::error::*;
use crate::parser::Rule;
use pest::iterators::Pair;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Declaration<'sc> {
    VariableDeclaration(VariableDeclaration<'sc>),
    FunctionDeclaration(FunctionDeclaration<'sc>),
    TraitDeclaration(TraitDeclaration<'sc>),
    StructDeclaration(StructDeclaration<'sc>),
    EnumDeclaration(EnumDeclaration<'sc>),
    Reassignment(Reassignment<'sc>),
    ImplTrait(ImplTrait<'sc>),
    ImplSelf(ImplSelf<'sc>),
    AbiDeclaration(AbiDeclaration<'sc>),
    ConstantDeclaration(ConstantDeclaration<'sc>),
}
impl<'sc> Declaration<'sc> {
    pub(crate) fn parse_from_pair(
        decl: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
        unassigned_docstring: String,
        docstrings: &mut HashMap<String, String>,
    ) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut pair = decl.clone().into_inner();
        let decl_inner = pair.next().unwrap();
        let parsed_declaration = match decl_inner.as_rule() {
            Rule::non_var_decl => check!(
                Self::parse_non_var_from_pair(decl_inner, config, unassigned_docstring, docstrings),
                return err(warnings, errors),
                warnings,
                errors
            ),
            Rule::var_decl => Declaration::VariableDeclaration(check!(
                VariableDeclaration::parse_from_pair(decl_inner, config, docstrings),
                return err(warnings, errors),
                warnings,
                errors
            )),
            Rule::reassignment => Declaration::Reassignment(check!(
                Reassignment::parse_from_pair(decl_inner, config, docstrings),
                return err(warnings, errors),
                warnings,
                errors
            )),
            a => unreachable!("declarations don't have any other sub-types: {:?}", a),
        };
        ok(parsed_declaration, warnings, errors)
    }

    pub(crate) fn parse_non_var_from_pair(
        decl: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
        unassigned_docstring: String,
        docstrings: &mut HashMap<String, String>,
    ) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut pair = decl.clone().into_inner();
        let decl_inner = pair.next().unwrap();
        let parsed_declaration = match decl_inner.as_rule() {
            Rule::fn_decl => {
                let fn_decl = check!(
                    FunctionDeclaration::parse_from_pair(decl_inner, config, docstrings),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                if !unassigned_docstring.is_empty() {
                    docstrings.insert(
                        format!("fn.{}", fn_decl.name.primary_name),
                        unassigned_docstring,
                    );
                }
                Declaration::FunctionDeclaration(fn_decl)
            }
            Rule::trait_decl => Declaration::TraitDeclaration(check!(
                TraitDeclaration::parse_from_pair(decl_inner, config, docstrings),
                return err(warnings, errors),
                warnings,
                errors
            )),
            Rule::struct_decl => {
                let struct_decl = check!(
                    StructDeclaration::parse_from_pair(decl_inner, config, docstrings),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                if !unassigned_docstring.is_empty() {
                    docstrings.insert(
                        format!("struct.{}", struct_decl.name.primary_name),
                        unassigned_docstring,
                    );
                }
                Declaration::StructDeclaration(struct_decl)
            }
            Rule::enum_decl => {
                let enum_decl = check!(
                    EnumDeclaration::parse_from_pair(decl_inner, config, docstrings),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                if !unassigned_docstring.is_empty() {
                    docstrings.insert(
                        format!("enum.{}", enum_decl.name.primary_name),
                        unassigned_docstring,
                    );
                }
                Declaration::EnumDeclaration(enum_decl)
            }
            Rule::impl_trait => Declaration::ImplTrait(check!(
                ImplTrait::parse_from_pair(decl_inner, config, docstrings),
                return err(warnings, errors),
                warnings,
                errors
            )),
            Rule::impl_self => Declaration::ImplSelf(check!(
                ImplSelf::parse_from_pair(decl_inner, config, docstrings),
                return err(warnings, errors),
                warnings,
                errors
            )),
            Rule::abi_decl => Declaration::AbiDeclaration(check!(
                AbiDeclaration::parse_from_pair(decl_inner, config, docstrings),
                return err(warnings, errors),
                warnings,
                errors
            )),
            Rule::const_decl => Declaration::ConstantDeclaration(check!(
                ConstantDeclaration::parse_from_pair(decl_inner, config, docstrings),
                return err(warnings, errors),
                warnings,
                errors
            )),
            a => unreachable!("declarations don't have any other sub-types: {:?}", a),
        };
        ok(parsed_declaration, warnings, errors)
    }
}
