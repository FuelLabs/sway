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
    pub fields: Vec<StructField>,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub visibility: Visibility,
    pub(crate) span: Span,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: Ident,
    pub(crate) r#type: TypeInfo,
    pub(crate) span: Span,
    pub(crate) type_span: Span,
}

impl StructDeclaration {
    pub(crate) fn parse_from_pair(
        decl: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let span = Span::from_pest(decl.as_span(), config.map(|x| x.path()));
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
        let name = check!(
            ident::parse_from_pair(name.expect("guaranteed to exist by grammar"), config),
            return err(warnings, errors),
            warnings,
            errors
        );
        assert_or_warn!(
            is_upper_camel_case(name.as_str()),
            warnings,
            name.span().clone(),
            Warning::NonClassCaseStructName {
                struct_name: name.clone()
            }
        );
        let type_parameters = check!(
            TypeParameter::parse_from_type_params_and_where_clause(
                type_params_pair,
                where_clause_pair,
                config,
            ),
            vec!(),
            warnings,
            errors
        );
        for type_parameter in type_parameters.iter() {
            assert_or_warn!(
                is_upper_camel_case(type_parameter.name_ident.as_str()),
                warnings,
                type_parameter.name_ident.span().clone(),
                Warning::NonClassCaseTypeParameter {
                    name: type_parameter.name_ident.clone()
                }
            );
        }
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
        ok(
            StructDeclaration {
                name,
                fields,
                type_parameters,
                visibility,
                span,
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
            let span = Span::from_pest(fields[i].as_span(), path.clone());
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
            let type_span = Span::from_pest(type_pair.as_span(), path.clone());
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
