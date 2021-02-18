use crate::ast::declaration::{TypeInfo, TypeParameter};
use crate::error::{CompileError, CompileResult, CompileWarning, Warning};
use crate::parser::{HllParser, Rule};
use inflector::cases::classcase::is_class_case;
use inflector::cases::snakecase::is_snake_case;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub(crate) struct StructDeclaration<'sc> {
    name: &'sc str,
    fields: Vec<StructField<'sc>>,
    type_parameters: Vec<TypeParameter<'sc>>,
}

#[derive(Debug, Clone)]
pub(crate) struct StructField<'sc> {
    name: &'sc str,
    r#type: TypeInfo<'sc>,
}

impl<'sc> StructDeclaration<'sc> {
    pub(crate) fn parse_from_pair(decl: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut decl = decl.into_inner();
        let name = decl.next().unwrap();
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
                _ => unreachable!(),
            }
        }

        let type_parameters = TypeParameter::parse_from_type_params_and_where_clause(
            type_params_pair,
            where_clause_pair,
        )?;

        let fields = if let Some(fields) = fields_pair {
            eval!(StructField::parse_from_pairs, warnings, fields)
        } else {
            Vec::new()
        };

        let span = name.as_span();
        let name = name.as_str();
        assert_or_warn!(
            is_class_case(name),
            warnings,
            span,
            Warning::NonClassCaseStructName { struct_name: name }
        );
        Ok((
            StructDeclaration {
                name,
                fields,
                type_parameters,
            },
            warnings,
        ))
    }
}

impl<'sc> StructField<'sc> {
    pub(crate) fn parse_from_pairs(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Vec<Self>> {
        let mut warnings = Vec::new();
        let mut fields = pair.into_inner().collect::<Vec<_>>();
        let mut fields_buf = Vec::new();
        for i in (0..fields.len()).step_by(2) {
            let span = fields[i].as_span();
            let name = fields[i].as_str();
            assert_or_warn!(
                is_snake_case(name),
                warnings,
                span,
                Warning::NonSnakeCaseStructFieldName { field_name: name }
            );
            let r#type = TypeInfo::parse_from_pair_inner(fields[i + 1].clone())?;
            fields_buf.push(StructField { name, r#type });
        }
        Ok((fields_buf, warnings))
    }
}
