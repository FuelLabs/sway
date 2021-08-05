use crate::parse_tree::declaration::TypeParameter;
use crate::parser::Rule;
use crate::types::TypeInfo;
use crate::{error::*, Ident};
use inflector::cases::classcase::is_class_case;
use inflector::cases::snakecase::is_snake_case;
use pest::iterators::Pair;
use pest::Span;

use super::Visibility;

#[derive(Debug, Clone)]
pub struct StructDeclaration<'sc> {
    pub name: Ident<'sc>,
    pub fields: Vec<StructField<'sc>>,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
    pub(crate) visibility: Visibility,
}

#[derive(Debug, Clone)]
pub struct StructField<'sc> {
    pub name: Ident<'sc>,
    pub(crate) r#type: TypeInfo<'sc>,
    pub(crate) span: Span<'sc>,
}

impl<'sc> StructDeclaration<'sc> {
    pub(crate) fn parse_from_pair(decl: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
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

        let type_parameters = match TypeParameter::parse_from_type_params_and_where_clause(
            type_params_pair,
            where_clause_pair,
        ) {
            CompileResult::Ok {
                value,
                warnings: mut l_w,
                errors: mut l_e,
            } => {
                warnings.append(&mut l_w);
                errors.append(&mut l_e);
                value
            }
            CompileResult::Err {
                warnings: mut l_w,
                errors: mut l_e,
            } => {
                warnings.append(&mut l_w);
                errors.append(&mut l_e);
                Vec::new()
            }
        };

        let fields = if let Some(fields) = fields_pair {
            eval!(
                StructField::parse_from_pairs,
                warnings,
                errors,
                fields,
                Vec::new()
            )
        } else {
            Vec::new()
        };

        let span = name.as_span();
        let name = eval!(
            Ident::parse_from_pair,
            warnings,
            errors,
            name,
            return err(warnings, errors)
        );
        assert_or_warn!(
            is_class_case(name.primary_name),
            warnings,
            span,
            Warning::NonClassCaseStructName {
                struct_name: name.primary_name
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

impl<'sc> StructField<'sc> {
    pub(crate) fn parse_from_pairs(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Vec<Self>> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let fields = pair.into_inner().collect::<Vec<_>>();
        let mut fields_buf = Vec::new();
        for i in (0..fields.len()).step_by(2) {
            let span = fields[i].as_span();
            let name = eval!(
                Ident::parse_from_pair,
                warnings,
                errors,
                fields[i],
                return err(warnings, errors)
            );
            assert_or_warn!(
                is_snake_case(name.primary_name),
                warnings,
                span.clone(),
                Warning::NonSnakeCaseStructFieldName {
                    field_name: name.primary_name.clone()
                }
            );
            let r#type = eval!(
                TypeInfo::parse_from_pair_inner,
                warnings,
                errors,
                fields[i + 1].clone(),
                TypeInfo::Unit
            );
            fields_buf.push(StructField { name, r#type, span });
        }
        ok(fields_buf, warnings, errors)
    }
}
