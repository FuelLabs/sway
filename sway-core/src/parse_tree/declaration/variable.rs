use crate::parser::Rule;
use crate::{
    error::{err, ok},
    ident,
    parse_tree::Expression,
    type_engine::TypeInfo,
    BuildConfig, CompileResult, Ident,
};

use pest::iterators::Pair;
use sway_types::span::Span;

#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    pub name: Ident,
    pub type_ascription: TypeInfo,
    pub type_ascription_span: Option<Span>,
    pub body: Expression, // will be codeblock variant
    pub is_mutable: bool,
}

impl VariableDeclaration {
    pub(crate) fn parse_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Vec<Self>> {
        let mut warnings = vec![];
        let mut errors = vec![];
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
                TypeInfo::Tuple(Vec::new()),
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
        let decl = VariableDeclaration {
            name: check!(
                ident::parse_from_pair(name_pair, config),
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
        };
        ok(vec![decl], warnings, errors)
    }
}
