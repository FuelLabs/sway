use crate::ast::declaration::TypeInfo;
use crate::error::{CompileError, CompileResult, CompileWarning, Warning};
use crate::parser::{HllParser, Rule};
use inflector::cases::classcase::is_class_case;
use inflector::cases::snakecase::is_snake_case;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub(crate) struct StructDeclaration<'sc> {
    name: &'sc str,
    fields: Vec<StructField<'sc>>,
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
        let fields = decl.next();
        let fields = if let Some(fields) = fields {
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
        Ok((StructDeclaration { name, fields }, warnings))
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
