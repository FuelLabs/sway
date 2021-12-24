mod abi;
mod constant;
mod r#enum;
pub mod function;
mod impl_trait;
mod reassignment;
mod storage;
mod r#struct;
mod r#trait;
mod type_parameter;
mod variable;

pub(crate) use abi::*;
pub(crate) use constant::*;
pub use function::*;
pub(crate) use impl_trait::*;
pub(crate) use r#enum::*;
pub use r#struct::*;
pub use r#trait::*;
pub(crate) use reassignment::*;
pub use storage::*;
pub(crate) use type_parameter::*;
pub use variable::*;

use crate::build_config::BuildConfig;
use crate::error::*;
use crate::parser::Rule;
use crate::*;
use fuel_pest::iterators::Pair;

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
    StorageDeclaration(StorageDeclaration<'sc>),
}
impl<'sc> Declaration<'sc> {
    pub(crate) fn parse_non_var_from_pair(
        decl: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut pair = decl.clone().into_inner();
        let decl_inner = pair.next().unwrap();
        let parsed_declaration = match decl_inner.as_rule() {
            Rule::non_var_decl => check!(
                Self::parse_non_var_from_pair(decl_inner, config),
                return err(warnings, errors),
                warnings,
                errors
            ),
            Rule::fn_decl => {
                let fn_decl = check!(
                    FunctionDeclaration::parse_from_pair(decl_inner, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                Declaration::FunctionDeclaration(fn_decl)
            }
            Rule::trait_decl => Declaration::TraitDeclaration(check!(
                TraitDeclaration::parse_from_pair(decl_inner, config,),
                return err(warnings, errors),
                warnings,
                errors
            )),
            Rule::struct_decl => {
                let struct_decl = check!(
                    StructDeclaration::parse_from_pair(decl_inner, config,),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                Declaration::StructDeclaration(struct_decl)
            }
            Rule::enum_decl => {
                let enum_decl = check!(
                    EnumDeclaration::parse_from_pair(decl_inner, config,),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                Declaration::EnumDeclaration(enum_decl)
            }
            Rule::impl_trait => Declaration::ImplTrait(check!(
                ImplTrait::parse_from_pair(decl_inner, config,),
                return err(warnings, errors),
                warnings,
                errors
            )),
            Rule::impl_self => Declaration::ImplSelf(check!(
                ImplSelf::parse_from_pair(decl_inner, config,),
                return err(warnings, errors),
                warnings,
                errors
            )),
            Rule::abi_decl => {
                let abi_decl = check!(
                    AbiDeclaration::parse_from_pair(decl_inner, config,),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                Declaration::AbiDeclaration(abi_decl)
            }
            Rule::const_decl => Declaration::ConstantDeclaration(check!(
                ConstantDeclaration::parse_from_pair(decl_inner, config,),
                return err(warnings, errors),
                warnings,
                errors
            )),
            Rule::storage_decl => Declaration::StorageDeclaration(check!(
                StorageDeclaration::parse_from_pair(decl_inner, config),
                return err(warnings, errors),
                warnings,
                errors
            )),
            a => unreachable!("declarations don't have any other sub-types: {:?}", a),
        };
        ok(parsed_declaration, warnings, errors)
    }
    pub(crate) fn parse_from_pair(
        decl: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut pair = decl.clone().into_inner();
        let decl_inner = pair.next().unwrap();
        let parsed_declaration = match decl_inner.as_rule() {
            Rule::fn_decl => Declaration::FunctionDeclaration(check!(
                FunctionDeclaration::parse_from_pair(decl_inner, config),
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
                let type_ascription_span = type_ascription
                    .clone()
                    .map(|x| x.into_inner().next().unwrap().as_span());
                let type_ascription = if let Some(ascription) = type_ascription {
                    let type_name = ascription.into_inner().next().unwrap();
                    check!(
                        TypeInfo::parse_from_pair(type_name, config),
                        TypeInfo::Unit,
                        warnings,
                        errors
                    )
                } else {
                    TypeInfo::Unknown
                };
                let body = check!(
                    Expression::parse_from_pair(maybe_body, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                Declaration::VariableDeclaration(VariableDeclaration {
                    name: check!(
                        Ident::parse_from_pair(name_pair, config),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ),
                    body,
                    is_mutable,
                    type_ascription,
                    type_ascription_span: type_ascription_span.map(|type_ascription_span| Span {
                        span: type_ascription_span,
                        path: config.map(|x| x.path()),
                    }),
                })
            }
            Rule::trait_decl => Declaration::TraitDeclaration(check!(
                TraitDeclaration::parse_from_pair(decl_inner, config),
                return err(warnings, errors),
                warnings,
                errors
            )),
            Rule::struct_decl => Declaration::StructDeclaration(check!(
                StructDeclaration::parse_from_pair(decl_inner, config),
                return err(warnings, errors),
                warnings,
                errors
            )),
            Rule::non_var_decl => check!(
                Self::parse_non_var_from_pair(decl_inner, config),
                return err(warnings, errors),
                warnings,
                errors
            ),
            Rule::enum_decl => Declaration::EnumDeclaration(check!(
                EnumDeclaration::parse_from_pair(decl_inner, config),
                return err(warnings, errors),
                warnings,
                errors
            )),
            Rule::reassignment => Declaration::Reassignment(check!(
                Reassignment::parse_from_pair(decl_inner, config),
                return err(warnings, errors),
                warnings,
                errors
            )),
            Rule::impl_trait => Declaration::ImplTrait(check!(
                ImplTrait::parse_from_pair(decl_inner, config),
                return err(warnings, errors),
                warnings,
                errors
            )),
            Rule::impl_self => Declaration::ImplSelf(check!(
                ImplSelf::parse_from_pair(decl_inner, config),
                return err(warnings, errors),
                warnings,
                errors
            )),
            Rule::const_decl => Declaration::ConstantDeclaration(check!(
                ConstantDeclaration::parse_from_pair(decl_inner, config),
                return err(warnings, errors),
                warnings,
                errors
            )),
            Rule::abi_decl => Declaration::AbiDeclaration(check!(
                AbiDeclaration::parse_from_pair(decl_inner, config),
                return err(warnings, errors),
                warnings,
                errors
            )),
            a => unreachable!("declarations don't have any other sub-types: {:?}", a),
        };
        ok(parsed_declaration, warnings, errors)
    }
}
