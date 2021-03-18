use crate::error::*;
use crate::parse_tree::declaration::TypeParameter;
use crate::parser::{HllParser, Rule};
use crate::types::TypeInfo;
use inflector::cases::classcase::is_class_case;
use inflector::cases::snakecase::is_snake_case;
use pest::iterators::Pair;
use pest::Span;
#[derive(Debug, Clone)]
pub struct EnumDeclaration<'sc> {
    pub(crate) name: &'sc str,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
    pub(crate) variants: Vec<EnumVariant<'sc>>,
    pub(crate) span: Span<'sc>,
}

#[derive(Debug, Clone)]
pub(crate) struct EnumVariant<'sc> {
    pub(crate) name: &'sc str,
    pub(crate) r#type: TypeInfo<'sc>,
}

impl<'sc> EnumDeclaration<'sc> {
    pub(crate) fn parse_from_pair(decl_inner: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let whole_enum_span = decl_inner.as_span();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut inner = decl_inner.into_inner();
        let _enum_keyword = inner.next().unwrap();
        let mut enum_name = None;
        let mut type_params = None;
        let mut where_clause = None;
        let mut variants = None;
        while let Some(pair) = inner.next() {
            match pair.as_rule() {
                Rule::enum_name => {
                    enum_name = Some(pair);
                }
                Rule::type_params => {
                    type_params = Some(pair);
                }
                Rule::trait_bounds => {
                    where_clause = Some(pair);
                }
                Rule::enum_fields => {
                    variants = Some(pair);
                }
                _ => unreachable!(),
            }
        }

        let type_parameters =
            match TypeParameter::parse_from_type_params_and_where_clause(type_params, where_clause)
            {
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

        // unwrap non-optional fields
        let enum_name = enum_name.unwrap();
        let span = enum_name.as_span();
        let name = enum_name.as_str();
        assert_or_warn!(
            is_class_case(name),
            warnings,
            span,
            Warning::NonClassCaseEnumName { enum_name: name }
        );

        let variants = eval!(
            EnumVariant::parse_from_pairs,
            warnings,
            errors,
            variants,
            Vec::new()
        );

        ok(
            EnumDeclaration {
                name,
                type_parameters,
                variants,
                span: whole_enum_span,
            },
            warnings,
            errors,
        )
    }
}

impl<'sc> EnumVariant<'sc> {
    pub(crate) fn parse_from_pairs(
        decl_inner: Option<Pair<'sc, Rule>>,
    ) -> CompileResult<'sc, Vec<Self>> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut fields_buf = Vec::new();
        if let Some(decl_inner) = decl_inner {
            let mut fields = decl_inner.into_inner().collect::<Vec<_>>();
            for i in (0..fields.len()).step_by(2) {
                let span = fields[i].as_span();
                let name = fields[i].as_str();
                assert_or_warn!(
                    is_class_case(name),
                    warnings,
                    span,
                    Warning::NonClassCaseEnumVariantName { variant_name: name }
                );
                let r#type = eval!(
                    TypeInfo::parse_from_pair_inner,
                    warnings,
                    errors,
                    fields[i + 1].clone(),
                    TypeInfo::Unit
                );
                fields_buf.push(EnumVariant { name, r#type });
            }
        }
        ok(fields_buf, warnings, errors)
    }
}
