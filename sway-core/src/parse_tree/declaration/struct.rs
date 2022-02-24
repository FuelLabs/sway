use crate::{
    build_config::BuildConfig,
    error::*,
    parse_tree::{declaration::TypeParameter, ident, Visibility},
    parser::Rule,
    style::{is_snake_case, is_upper_camel_case},
    type_engine::TypeInfo,
};

use sway_types::{ident::Ident, span::Span};

use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct StructDeclaration {
    pub name: Ident,
    pub(crate) fields: Vec<StructField>,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub visibility: Visibility,
}

#[derive(Debug, Clone)]
pub(crate) struct StructField {
    pub(crate) name: Ident,
    pub(crate) r#type: TypeInfo,
    pub(crate) span: Span,
    pub(crate) type_span: Span,
}

impl StructDeclaration {
    pub(crate) fn parse_from_pair(
        decl: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Self> {
        let path = config.map(|c| c.path());
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let decl = decl.into_inner();
        let mut visibility = Visibility::Private;
        let mut name = None;
        let mut type_params_pair = None;
        let mut where_clause_pair = None;
        let mut fields_pair = None;
        for pair in decl {
            match pair.as_rule() {
                Rule::type_params => {
                    type_params_pair = Some(pair);
                }
                Rule::trait_bounds => {
                    where_clause_pair = Some(pair);
                }
                Rule::struct_fields => {
                    fields_pair = Some(pair);
                }
                Rule::struct_keyword => (),
                Rule::struct_name => {
                    name = Some(pair);
                }
                Rule::visibility => {
                    visibility = Visibility::parse_from_pair(pair);
                }
                a => unreachable!("{:?}", a),
            }
        }
        let name = name.expect("guaranteed to exist by grammar");

        let type_parameters = TypeParameter::parse_from_type_params_and_where_clause(
            type_params_pair,
            where_clause_pair,
            config,
        )
        .unwrap_or_else(&mut warnings, &mut errors, Vec::new);

        let fields = if let Some(fields) = fields_pair {
            check!(
                StructField::parse_from_pairs(fields, config),
                Vec::new(),
                warnings,
                errors
            )
        } else {
            Vec::new()
        };

        let span = Span {
            span: name.as_span(),
            path,
        };

        let name = check!(
            ident::parse_from_pair(name, config),
            return err(warnings, errors),
            warnings,
            errors
        );
        assert_or_warn!(
            is_upper_camel_case(name.as_str()),
            warnings,
            span,
            Warning::NonClassCaseStructName {
                struct_name: name.clone()
            }
        );
        ok(
            StructDeclaration {
                name,
                fields,
                type_parameters,
                visibility,
            },
            warnings,
            errors,
        )
    }
}

impl StructField {
    pub(crate) fn parse_from_pairs(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Vec<Self>> {
        let path = config.map(|c| c.path());
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let fields = pair.into_inner().collect::<Vec<_>>();
        let mut fields_buf = Vec::new();
        for i in (0..fields.len()).step_by(2) {
            let span = Span {
                span: fields[i].as_span(),
                path: path.clone(),
            };
            let name = check!(
                ident::parse_from_pair(fields[i].clone(), config),
                return err(warnings, errors),
                warnings,
                errors
            );
            assert_or_warn!(
                is_snake_case(name.as_str()),
                warnings,
                span.clone(),
                Warning::NonSnakeCaseStructFieldName {
                    field_name: name.clone(),
                }
            );
            let type_pair = fields[i + 1].clone();
            let type_span = Span {
                span: type_pair.as_span(),
                path: path.clone(),
            };
            let r#type = check!(
                TypeInfo::parse_from_pair(fields[i + 1].clone(), config),
                TypeInfo::Tuple(Vec::new()),
                warnings,
                errors
            );
            fields_buf.push(StructField {
                name,
                r#type,
                span,
                type_span,
            });
        }
        ok(fields_buf, warnings, errors)
    }
}
