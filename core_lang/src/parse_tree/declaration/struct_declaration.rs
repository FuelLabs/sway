use crate::build_config::BuildConfig;
use crate::parse_tree::declaration::TypeParameter;
use crate::parser::Rule;
use crate::span::Span;
use crate::type_engine::TypeInfo;
use crate::{error::*, Ident};
use inflector::cases::classcase::is_class_case;
use inflector::cases::snakecase::is_snake_case;
use pest::iterators::Pair;
use std::collections::HashMap;

use super::Visibility;

#[derive(Debug, Clone)]
pub struct StructDeclaration<'sc> {
    pub name: Ident<'sc>,
    pub(crate) fields: Vec<StructField<'sc>>,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
    pub visibility: Visibility,
}

#[derive(Debug, Clone)]
pub(crate) struct StructField<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) r#type: TypeInfo,
    pub(crate) span: Span<'sc>,
}

impl<'sc> StructDeclaration<'sc> {
    pub(crate) fn parse_from_pair(
        decl: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
        docstrings: &mut HashMap<String, String>,
    ) -> CompileResult<'sc, Self> {
        let path = config.map(|c| c.path());
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut decl = decl.into_inner();
        let mut visibility = Visibility::Private;
        let mut name = None;
        let mut type_params_pair = None;
        let mut where_clause_pair = None;
        let mut fields_pair = None;
        while let Some(pair) = decl.next() {
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
        .unwrap_or_else(&mut warnings, &mut errors, || Vec::new());

        let span = Span {
            span: name.as_span(),
            path: path.clone(),
        };
        let name = check!(
            Ident::parse_from_pair(name, config),
            return err(warnings, errors),
            warnings,
            errors
        );
        assert_or_warn!(
            is_class_case(name.primary_name),
            warnings,
            span,
            Warning::NonClassCaseStructName {
                struct_name: name.primary_name
            }
        );
        let fields = if let Some(fields) = fields_pair {
            check!(
                StructField::parse_from_pairs(
                    fields,
                    config,
                    name.primary_name.to_string(),
                    docstrings
                ),
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
            },
            warnings,
            errors,
        )
    }
}

impl<'sc> StructField<'sc> {
    pub(crate) fn parse_from_pairs(
        pair: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
        struct_name: String,
        docstrings: &mut HashMap<String, String>,
    ) -> CompileResult<'sc, Vec<Self>> {
        let path = config.map(|c| c.path());
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let fields = pair.into_inner().collect::<Vec<_>>();
        let mut fields_buf = Vec::new();
        let mut unassigned_docstring = "".to_string();
        let mut i = 0;
        while i < fields.len() {
            let field = &fields[i];
            match field.as_rule() {
                Rule::docstring => {
                    let docstring = field.as_str().to_string().split_off(3);
                    let docstring = docstring.as_str().trim();
                    unassigned_docstring.push_str("\n");
                    unassigned_docstring.push_str(docstring);
                    i = i + 1;
                }
                _ => {
                    let span = Span {
                        span: fields[i].as_span(),
                        path: path.clone(),
                    };
                    let name = check!(
                        Ident::parse_from_pair(fields[i].clone(), config),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    if !unassigned_docstring.is_empty() {
                        docstrings.insert(
                            format!("struct.{}.{}", struct_name, name.primary_name),
                            unassigned_docstring.clone(),
                        );
                        unassigned_docstring.clear();
                    }
                    assert_or_warn!(
                        is_snake_case(name.primary_name),
                        warnings,
                        span.clone(),
                        Warning::NonSnakeCaseStructFieldName {
                            field_name: name.primary_name
                        }
                    );
                    let r#type = check!(
                        TypeInfo::parse_from_pair_inner(fields[i + 1].clone(), config),
                        TypeInfo::Unit,
                        warnings,
                        errors
                    );
                    fields_buf.push(StructField { name, r#type, span });
                    i = i + 2;
                }
            }
        }
        ok(fields_buf, warnings, errors)
    }
}
