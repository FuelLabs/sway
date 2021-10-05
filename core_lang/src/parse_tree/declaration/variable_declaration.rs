use crate::parse_tree::Expression;
use crate::{types::TypeInfo, Ident};

use crate::build_config::BuildConfig;
use crate::error::{err, ok, CompileResult};
use crate::parser::Rule;
use pest::iterators::Pair;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct VariableDeclaration<'sc> {
    pub name: Ident<'sc>,
    pub type_ascription: TypeInfo<'sc>,
    pub body: Expression<'sc>, // will be codeblock variant
    pub is_mutable: bool,
}

impl<'sc> VariableDeclaration<'sc> {
    pub(crate) fn parse_from_pair(
        pair: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
        docstrings: &mut HashMap<String, String>,
    ) -> CompileResult<'sc, VariableDeclaration<'sc>> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut var_decl_parts = pair.into_inner();
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
                type_asc
            }
            _ => TypeInfo::Unknown,
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
        ok(
            VariableDeclaration {
                name: check!(
                    Ident::parse_from_pair(name_pair, config.clone()),
                    return err(warnings, errors),
                    warnings,
                    errors
                ),
                body,
                is_mutable,
                type_ascription,
            },
            warnings,
            errors,
        )
    }
}
