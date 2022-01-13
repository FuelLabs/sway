use crate::{
    build_config::BuildConfig,
    error::*,
    parse_tree::{declaration::TypeParameter, ident, Visibility},
    parser::Rule,
    semantic_analysis::ast_node::{
        declaration::insert_type_parameters, TypedEnumDeclaration, TypedEnumVariant,
    },
    style::is_upper_camel_case,
    type_engine::*,
    Namespace,
};

use sway_types::{ident::Ident, span::Span};

use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct EnumDeclaration {
    pub name: Ident,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub(crate) variants: Vec<EnumVariant>,
    pub(crate) span: Span,
    pub visibility: Visibility,
}

#[derive(Debug, Clone)]
pub(crate) struct EnumVariant {
    pub(crate) name: Ident,
    pub(crate) r#type: TypeInfo,
    pub(crate) tag: usize,
    pub(crate) span: Span,
}

impl EnumDeclaration {
    /// Looks up the various TypeInfos in the [Namespace] to see if they are generic or refer to
    /// something.
    pub(crate) fn to_typed_decl(
        &self,
        namespace: &mut Namespace,
        self_type: TypeId,
    ) -> TypedEnumDeclaration {
        let mut variants_buf = vec![];
        let mut errors = vec![];
        let mut warnings = vec![];

        let type_mapping = insert_type_parameters(&self.type_parameters);
        for variant in &self.variants {
            variants_buf.push(check!(
                variant.to_typed_decl(namespace, self_type, variant.span.clone(), &type_mapping),
                continue,
                warnings,
                errors
            ));
        }
        TypedEnumDeclaration {
            name: self.name.clone(),
            type_parameters: self.type_parameters.clone(),
            variants: variants_buf,
            span: self.span.clone(),
            visibility: self.visibility,
        }
    }

    pub(crate) fn parse_from_pair(
        decl_inner: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Self> {
        let path = config.map(|c| c.path());
        let whole_enum_span = Span {
            span: decl_inner.as_span(),
            path: path.clone(),
        };
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let inner = decl_inner.into_inner();
        let mut visibility = Visibility::Private;
        let mut enum_name = None;
        let mut type_params = None;
        let mut where_clause = None;
        let mut variants = None;
        for pair in inner {
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
                Rule::enum_keyword => (),
                Rule::visibility => {
                    visibility = Visibility::parse_from_pair(pair);
                }
                _ => unreachable!(),
            }
        }

        let type_parameters = TypeParameter::parse_from_type_params_and_where_clause(
            type_params,
            where_clause,
            config,
        )
        .unwrap_or_else(&mut warnings, &mut errors, Vec::new);

        // unwrap non-optional fields
        let enum_name = enum_name.unwrap();
        let name = check!(
            ident::parse_from_pair(enum_name.clone(), config),
            return err(warnings, errors),
            warnings,
            errors
        );
        assert_or_warn!(
            is_upper_camel_case(name.as_str()),
            warnings,
            Span {
                span: enum_name.as_span(),
                path,
            },
            Warning::NonClassCaseEnumName {
                enum_name: name.clone()
            }
        );

        let variants = check!(
            EnumVariant::parse_from_pairs(variants, config),
            Vec::new(),
            warnings,
            errors
        );

        ok(
            EnumDeclaration {
                name,
                type_parameters,
                variants,
                span: whole_enum_span,
                visibility,
            },
            warnings,
            errors,
        )
    }
}

impl EnumVariant {
    pub(crate) fn to_typed_decl(
        &self,
        namespace: &mut Namespace,
        self_type: TypeId,
        span: Span,
        type_mapping: &[(TypeParameter, TypeId)],
    ) -> CompileResult<TypedEnumVariant> {
        let mut errors = vec![];
        let enum_variant_type =
            if let Some(matching_id) = self.r#type.matches_type_parameter(type_mapping) {
                insert_type(TypeInfo::Ref(matching_id))
            } else {
                namespace
                    .resolve_type_with_self(self.r#type.clone(), self_type)
                    .unwrap_or_else(|_| {
                        errors.push(CompileError::UnknownType { span });
                        insert_type(TypeInfo::ErrorRecovery)
                    })
            };
        ok(
            TypedEnumVariant {
                name: self.name.clone(),
                r#type: enum_variant_type,
                tag: self.tag,
                span: self.span.clone(),
            },
            vec![],
            errors,
        )
    }
    pub(crate) fn parse_from_pairs(
        decl_inner: Option<Pair<Rule>>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Vec<Self>> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut fields_buf = Vec::new();
        let mut tag = 0;
        if let Some(decl_inner) = decl_inner {
            let fields = decl_inner.into_inner().collect::<Vec<_>>();
            for i in (0..fields.len()).step_by(2) {
                let variant_span = Span {
                    span: fields[i].as_span(),
                    path: config.map(|c| c.path()),
                };
                let name = check!(
                    ident::parse_from_pair(fields[i].clone(), config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                assert_or_warn!(
                    is_upper_camel_case(name.as_str()),
                    warnings,
                    name.span().clone(),
                    Warning::NonClassCaseEnumVariantName {
                        variant_name: name.clone()
                    }
                );
                let r#type = check!(
                    TypeInfo::parse_from_pair(fields[i + 1].clone(), config),
                    TypeInfo::Tuple(Vec::new()),
                    warnings,
                    errors
                );
                fields_buf.push(EnumVariant {
                    name,
                    r#type,
                    tag,
                    span: variant_span,
                });
                tag += 1;
            }
        }
        ok(fields_buf, warnings, errors)
    }
}
