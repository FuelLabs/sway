mod function_declaration;
mod struct_declaration;
mod trait_declaration;
mod variable_declaration;

pub(crate) use function_declaration::*;
pub(crate) use struct_declaration::*;
pub(crate) use trait_declaration::*;
pub(crate) use variable_declaration::*;

use crate::ast::Expression;
use crate::error::CompileError;
use crate::parser::{HllParser, Rule};
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub(crate) enum Declaration<'sc> {
    VariableDeclaration(VariableDeclaration<'sc>),
    FunctionDeclaration(FunctionDeclaration<'sc>),
    TraitDeclaration(TraitDeclaration<'sc>),
    StructDeclaration(StructDeclaration<'sc>),
}
impl<'sc> Declaration<'sc> {
    pub(crate) fn parse_from_pair(decl: Pair<'sc, Rule>) -> Result<Self, CompileError<'sc>> {
        let mut pair = decl.clone().into_inner();
        let decl_inner = pair.next().unwrap();
        let parsed_declaration = match decl_inner.as_rule() {
            Rule::fn_decl => {
                Declaration::FunctionDeclaration(FunctionDeclaration::parse_from_pair(decl_inner)?)
            }
            Rule::var_decl => {
                let mut var_decl_parts = decl_inner.into_inner();
                let _let_keyword = var_decl_parts.next();
                let name: &'sc str = var_decl_parts.next().unwrap().as_str().trim();
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
                let body = Expression::parse_from_pair(maybe_body)?;
                Declaration::VariableDeclaration(VariableDeclaration {
                    name,
                    body,
                    type_ascription,
                })
            }
            Rule::trait_decl => {
                Declaration::TraitDeclaration(TraitDeclaration::parse_from_pair(decl_inner)?)
            }
            Rule::struct_decl => {
                Declaration::StructDeclaration(StructDeclaration::parse_from_pair(decl_inner)?)
            }
            _ => unreachable!("declarations don't have any other sub-types"),
        };
        Ok(parsed_declaration)
    }
}

// option res to res option helper
fn invert<T, E>(x: Option<Result<T, E>>) -> Result<Option<T>, E> {
    x.map_or(Ok(None), |v| v.map(Some))
}
