mod enum_declaration;
mod function_declaration;
mod struct_declaration;
mod trait_declaration;
mod type_parameter;
mod variable_declaration;

pub(crate) use enum_declaration::*;
pub(crate) use function_declaration::*;
pub(crate) use struct_declaration::*;
pub(crate) use trait_declaration::*;
pub(crate) use type_parameter::*;
pub(crate) use variable_declaration::*;

use crate::error::{ParseError, ParseResult};
use crate::parse_tree::{Expression, VarName};
use crate::parser::{HllParser, Rule};
use crate::types::TypeInfo;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub(crate) enum Declaration<'sc> {
    VariableDeclaration(VariableDeclaration<'sc>),
    FunctionDeclaration(FunctionDeclaration<'sc>),
    TraitDeclaration(TraitDeclaration<'sc>),
    StructDeclaration(StructDeclaration<'sc>),
    EnumDeclaration(EnumDeclaration<'sc>),
}
impl<'sc> Declaration<'sc> {
    pub(crate) fn parse_from_pair(decl: Pair<'sc, Rule>) -> ParseResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut pair = decl.clone().into_inner();
        let decl_inner = pair.next().unwrap();
        let parsed_declaration = match decl_inner.as_rule() {
            Rule::fn_decl => Declaration::FunctionDeclaration(eval!(
                FunctionDeclaration::parse_from_pair,
                warnings,
                decl_inner
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
                let type_ascription =
                    invert(type_ascription.map(|x| TypeInfo::parse_from_pair(x)))?;
                let body = eval!(Expression::parse_from_pair, warnings, maybe_body);
                Declaration::VariableDeclaration(VariableDeclaration {
                    name: VarName::parse_from_pair(name_pair)?,
                    body,
                    is_mutable,
                    type_ascription,
                })
            }
            Rule::trait_decl => Declaration::TraitDeclaration(eval!(
                TraitDeclaration::parse_from_pair,
                warnings,
                decl_inner
            )),
            Rule::struct_decl => Declaration::StructDeclaration(eval!(
                StructDeclaration::parse_from_pair,
                warnings,
                decl_inner
            )),
            Rule::enum_decl => Declaration::EnumDeclaration(eval!(
                EnumDeclaration::parse_from_pair,
                warnings,
                decl_inner
            )),
            a => unreachable!("declarations don't have any other sub-types: {:?}", a),
        };
        Ok((parsed_declaration, warnings))
    }
}

// option res to res option helper
fn invert<T, E>(x: Option<Result<T, E>>) -> Result<Option<T>, E> {
    x.map_or(Ok(None), |v| v.map(Some))
}
