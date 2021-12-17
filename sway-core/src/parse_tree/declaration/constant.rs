use crate::parse_tree::{Expression, Visibility};
use crate::{type_engine::TypeInfo, Ident};

use crate::build_config::BuildConfig;
use crate::error::{err, ok, CompileResult, Warning};
use crate::parser::Rule;
use crate::span::Span;
use crate::style::is_screaming_snake_case;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct ConstantDeclaration<'sc> {
    pub name: Ident<'sc>,
    pub type_ascription: TypeInfo,
    pub value: Expression<'sc>,
    pub visibility: Visibility,
}

impl<'sc> ConstantDeclaration<'sc> {
    pub(crate) fn parse_from_pair(
        pair: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, ConstantDeclaration<'sc>> {
        let path = config.map(|c| c.path());
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut const_decl_parts = pair.into_inner();
        let visibility = match const_decl_parts.next().unwrap().as_rule() {
            Rule::const_decl_keyword => Visibility::Private,
            Rule::visibility => {
                let _const_keyword = const_decl_parts.next();
                Visibility::Public
            }
            _ => unreachable!(),
        };
        let name_pair = const_decl_parts.next().unwrap();
        let mut maybe_value = const_decl_parts.next().unwrap();
        let type_ascription = match maybe_value.as_rule() {
            Rule::type_ascription => {
                let type_asc = maybe_value.clone();
                maybe_value = const_decl_parts.next().unwrap();
                Some(type_asc)
            }
            _ => None,
        };
        let type_ascription = type_ascription
            .map(|ascription| {
                let type_name = ascription.into_inner().next().unwrap();
                check!(
                    TypeInfo::parse_from_pair(type_name, config),
                    TypeInfo::Tuple(Vec::new()),
                    warnings,
                    errors
                )
            })
            .unwrap_or(TypeInfo::Unknown);
        let value = check!(
            Expression::parse_from_pair_inner(maybe_value, config),
            return err(warnings, errors),
            warnings,
            errors
        );
        let name = check!(
            Ident::parse_from_pair(name_pair.clone(), config),
            return err(warnings, errors),
            warnings,
            errors
        );
        assert_or_warn!(
            is_screaming_snake_case(name.primary_name()),
            warnings,
            Span {
                span: name_pair.as_span(),
                path,
            },
            Warning::NonScreamingSnakeCaseConstName {
                name: name.primary_name(),
            }
        );
        ok(
            ConstantDeclaration {
                name,
                type_ascription,
                value,
                visibility,
            },
            warnings,
            errors,
        )
    }
}
