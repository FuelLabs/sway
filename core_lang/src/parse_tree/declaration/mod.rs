mod abi_declaration;
mod enum_declaration;
pub mod function_declaration;
mod impl_trait;
mod reassignment;
mod struct_declaration;
mod trait_declaration;
mod type_parameter;
mod variable_declaration;

pub(crate) use abi_declaration::*;
pub(crate) use enum_declaration::*;
pub(crate) use function_declaration::*;
pub(crate) use impl_trait::*;
pub(crate) use reassignment::*;
pub(crate) use struct_declaration::*;
pub(crate) use trait_declaration::*;
pub(crate) use type_parameter::*;
pub use variable_declaration::*;

use crate::build_config::BuildConfig;
use crate::error::*;
use crate::parse_tree::Expression;
use crate::parser::Rule;
use crate::types::TypeInfo;
use crate::Ident;
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
}
impl<'sc> Declaration<'sc> {
    pub(crate) fn parse_from_pair(
        decl: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
        unassigned_docstrings: Vec<String>,
        docstrings: &mut HashMap<String, Vec<String>>
    ) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut pair = decl.clone().into_inner();
        let decl_inner = pair.next().unwrap();
        let parsed_declaration = match decl_inner.as_rule() {
            Rule::fn_decl => Declaration::FunctionDeclaration(check!(
                FunctionDeclaration::parse_from_pair(decl_inner, config, docstrings),
                return err(warnings, errors),
                warnings,
                errors
            )),
            Rule::var_decl => {
                let mut var_decl_parts = decl_inner.into_inner();
                let _let_keyword = var_decl_parts.next();
                let maybe_mut_keyword = var_decl_parts.next().unwrap();
                let is_mutable = maybe_mut_keyword.as_rule() == Rule::mut_keyword;
                let name_pair = if is_mutable {
                    var_decl_parts.next().unwrap()
                } else {
                    maybe_mut_keyword
                };
                let mut maybe_body = var_decl_parts.next().unwrap();
                let type_ascription = match maybe_body.as_rule() {
                    Rule::type_ascription => {
                        let type_asc = maybe_body.clone();
                        maybe_body = var_decl_parts.next().unwrap();
                        Some(type_asc)
                    }
                    _ => None,
                };
                let type_ascription = if let Some(ascription) = type_ascription {
                    Some(check!(
                        TypeInfo::parse_from_pair(ascription, config.clone()),
                        TypeInfo::Unit,
                        warnings,
                        errors
                    ))
                } else {
                    None
                };
                let body = check!(
                    Expression::parse_from_pair(maybe_body, config.clone(), docstrings),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                Declaration::VariableDeclaration(VariableDeclaration {
                    name: check!(
                        Ident::parse_from_pair(name_pair, config.clone()),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ),
                    body,
                    is_mutable,
                    type_ascription,
                })
            }
            Rule::trait_decl => Declaration::TraitDeclaration(check!(
                TraitDeclaration::parse_from_pair(decl_inner, config, docstrings),
                return err(warnings, errors),
                warnings,
                errors
            )),
            Rule::struct_decl => {
                let struct_decl = check!(
                    StructDeclaration::parse_from_pair(decl_inner, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                if unassigned_docstrings.len() > 0 {
                    docstrings.insert(
                        format!("struct.{}", struct_decl.name.primary_name),
                        unassigned_docstrings
                    );
                }
                Declaration::StructDeclaration(struct_decl)
            },
            Rule::enum_decl => {
                let enum_decl = check!(
                    EnumDeclaration::parse_from_pair(decl_inner, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                if unassigned_docstrings.len() > 0 {
                    docstrings.insert(
                        format!("enum.{}", enum_decl.name.primary_name),
                        unassigned_docstrings
                    );
                }
                Declaration::EnumDeclaration(enum_decl)
            },
            Rule::reassignment => Declaration::Reassignment(check!(
                Reassignment::parse_from_pair(decl_inner, config, docstrings),
                return err(warnings, errors),
                warnings,
                errors
            )),
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
            a => unreachable!("declarations don't have any other sub-types: {:?}", a),
        };
        ok(parsed_declaration, warnings, errors)
    }
}
