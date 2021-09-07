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

use crate::error::*;
use crate::parse_tree::Expression;
use crate::parser::Rule;
use crate::types::TypeInfo;
use crate::Ident;
use pest::iterators::Pair;
use crate::build_config::BuildConfig;

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
    pub(crate) fn parse_from_pair(decl: Pair<'sc, Rule>, config: Option<BuildConfig>) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut pair = decl.clone().into_inner();
        let decl_inner = pair.next().unwrap();
        let parsed_declaration = match decl_inner.as_rule() {
            Rule::fn_decl => Declaration::FunctionDeclaration(eval2!(
                FunctionDeclaration::parse_from_pair,
                warnings,
                errors,
                decl_inner,
                config,
                return err(warnings, errors)
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
                    Some(eval2!(
                        TypeInfo::parse_from_pair,
                        warnings,
                        errors,
                        ascription,
                        config,
                        TypeInfo::Unit
                    ))
                } else {
                    None
                };
                let body = eval2!(
                    Expression::parse_from_pair,
                    warnings,
                    errors,
                    maybe_body,
                    config,
                    return err(warnings, errors)
                );
                Declaration::VariableDeclaration(VariableDeclaration {
                    name: eval2!(
                        Ident::parse_from_pair,
                        warnings,
                        errors,
                        name_pair,
                        config,
                        return err(warnings, errors)
                    ),
                    body,
                    is_mutable,
                    type_ascription,
                })
            }
            Rule::trait_decl => Declaration::TraitDeclaration(eval2!(
                TraitDeclaration::parse_from_pair,
                warnings,
                errors,
                decl_inner,
                config,
                return err(warnings, errors)
            )),
            Rule::struct_decl => Declaration::StructDeclaration(eval2!(
                StructDeclaration::parse_from_pair,
                warnings,
                errors,
                decl_inner,
                config,
                return err(warnings, errors)
            )),
            Rule::enum_decl => Declaration::EnumDeclaration(eval2!(
                EnumDeclaration::parse_from_pair,
                warnings,
                errors,
                decl_inner,
                config,
                return err(warnings, errors)
            )),
            Rule::reassignment => Declaration::Reassignment(eval2!(
                Reassignment::parse_from_pair,
                warnings,
                errors,
                decl_inner,
                config,
                return err(warnings, errors)
            )),
            Rule::impl_trait => Declaration::ImplTrait(eval2!(
                ImplTrait::parse_from_pair,
                warnings,
                errors,
                decl_inner,
                config,
                return err(warnings, errors)
            )),
            Rule::impl_self => Declaration::ImplSelf(eval2!(
                ImplSelf::parse_from_pair,
                warnings,
                errors,
                decl_inner,
                config,
                return err(warnings, errors)
            )),
            Rule::abi_decl => Declaration::AbiDeclaration(eval2!(
                AbiDeclaration::parse_from_pair,
                warnings,
                errors,
                decl_inner,
                config,
                return err(warnings, errors)
            )),
            a => unreachable!("declarations don't have any other sub-types: {:?}", a),
        };
        ok(parsed_declaration, warnings, errors)
    }
}
