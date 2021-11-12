use crate::build_config::BuildConfig;
use crate::parser::Rule;
use crate::span::Span;
use crate::type_engine::{TypeId, TypeInfo};

use crate::Ident;
use crate::Namespace;
use crate::{error::*, semantic_analysis::ast_node::TypedEnumDeclaration};
use crate::{
    parse_tree::declaration::TypeParameter, semantic_analysis::ast_node::TypedEnumVariant,
};
use inflector::cases::classcase::is_class_case;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct EnumDeclaration<'sc> {
    pub name: Ident<'sc>,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
    pub(crate) variants: Vec<EnumVariant<'sc>>,
    pub(crate) span: Span<'sc>,
}

#[derive(Debug, Clone)]
pub(crate) struct EnumVariant<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) r#type: TypeInfo,
    pub(crate) tag: usize,
    pub(crate) span: Span<'sc>,
}

impl<'sc> EnumDeclaration<'sc> {
    /// Looks up the various TypeInfos in the [Namespace] to see if they are generic or refer to
    /// something.
    pub(crate) fn to_typed_decl(
        &self,
        namespace: &mut Namespace<'sc>,
        self_type: TypeId,
    ) -> TypedEnumDeclaration<'sc> {
        let mut variants_buf = vec![];
        let mut errors = vec![];
        let mut warnings = vec![];

        for variant in &self.variants {
            variants_buf.push(check!(
                variant.to_typed_decl(namespace, self_type, variant.span.clone()),
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
        }
    }

    pub(crate) fn parse_from_pair(
        decl_inner: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        let path = config.map(|c| c.path());
        let whole_enum_span = Span {
            span: decl_inner.as_span(),
            path: path.clone(),
        };
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut inner = decl_inner.into_inner();
        let _enum_keyword = inner.next().unwrap();
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
            Ident::parse_from_pair(enum_name.clone(), config),
            return err(warnings, errors),
            warnings,
            errors
        );
        assert_or_warn!(
            is_class_case(name.primary_name),
            warnings,
            Span {
                span: enum_name.as_span(),
                path
            },
            Warning::NonClassCaseEnumName {
                enum_name: name.primary_name
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
            },
            warnings,
            errors,
        )
    }
}

impl<'sc> EnumVariant<'sc> {
    pub(crate) fn to_typed_decl(
        &self,
        namespace: &mut Namespace<'sc>,
        self_type: TypeId,
        _span: Span<'sc>,
    ) -> CompileResult<'sc, TypedEnumVariant<'sc>> {
        ok(
            TypedEnumVariant {
                name: self.name.clone(),
                r#type: namespace.resolve_type_with_self(self.r#type.clone(), self_type),
                tag: self.tag,
                span: self.span.clone(),
            },
            vec![],
            vec![],
        )
    }
    pub(crate) fn parse_from_pairs(
        decl_inner: Option<Pair<'sc, Rule>>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Vec<Self>> {
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
                    Ident::parse_from_pair(fields[i].clone(), config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                assert_or_warn!(
                    is_class_case(name.primary_name),
                    warnings,
                    name.span.clone(),
                    Warning::NonClassCaseEnumVariantName {
                        variant_name: name.primary_name
                    }
                );
                let r#type = check!(
                    TypeInfo::parse_from_pair_inner(fields[i + 1].clone(), config),
                    TypeInfo::Unit,
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
