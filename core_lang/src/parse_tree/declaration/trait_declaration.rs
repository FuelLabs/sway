use super::{FunctionDeclaration, FunctionParameter, Visibility};
use crate::build_config::BuildConfig;
use crate::parse_tree::TypeParameter;
use crate::parser::Rule;
use crate::span::Span;
use crate::type_engine::TypeInfo;
use crate::{error::*, Ident};
use inflector::cases::classcase::is_class_case;
use inflector::cases::snakecase::is_snake_case;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct TraitDeclaration<'sc> {
    pub name: Ident<'sc>,
    pub(crate) interface_surface: Vec<TraitFn<'sc>>,
    pub(crate) methods: Vec<FunctionDeclaration<'sc>>,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
    pub visibility: Visibility,
}

impl<'sc> TraitDeclaration<'sc> {
    pub(crate) fn parse_from_pair(
        pair: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut trait_parts = pair.into_inner().peekable();
        let trait_keyword_or_visibility = trait_parts.next().unwrap();
        let (visibility, _trait_keyword) =
            if trait_keyword_or_visibility.as_rule() == Rule::visibility {
                (
                    Visibility::parse_from_pair(trait_keyword_or_visibility),
                    trait_parts.next().unwrap(),
                )
            } else {
                (Visibility::Private, trait_keyword_or_visibility)
            };
        let name_pair = trait_parts.next().unwrap();
        let name = check!(
            Ident::parse_from_pair(name_pair.clone(), config),
            return err(warnings, errors),
            warnings,
            errors
        );
        let span = name.span.clone();
        assert_or_warn!(
            is_class_case(name_pair.as_str().trim()),
            warnings,
            span,
            Warning::NonClassCaseTraitName {
                name: name_pair.as_str().trim()
            }
        );
        let mut type_params_pair = None;
        let mut where_clause_pair = None;
        let mut methods = Vec::new();
        let mut interface = Vec::new();

        for _ in 0..2 {
            match trait_parts.peek().map(|x| x.as_rule()) {
                Some(Rule::trait_bounds) => {
                    where_clause_pair = Some(trait_parts.next().unwrap());
                }
                Some(Rule::type_params) => {
                    type_params_pair = Some(trait_parts.next().unwrap());
                }
                _ => (),
            }
        }

        if let Some(methods_and_interface) = trait_parts.next() {
            for fn_sig_or_decl in methods_and_interface.into_inner() {
                match fn_sig_or_decl.as_rule() {
                    Rule::fn_signature => {
                        interface.push(check!(
                            TraitFn::parse_from_pair(fn_sig_or_decl, config),
                            continue,
                            warnings,
                            errors
                        ));
                    }
                    Rule::fn_decl => {
                        methods.push(check!(
                            FunctionDeclaration::parse_from_pair(fn_sig_or_decl, config),
                            continue,
                            warnings,
                            errors
                        ));
                    }
                    a => unreachable!("{:?}", a),
                }
            }
        }
        let type_parameters =
            crate::parse_tree::declaration::TypeParameter::parse_from_type_params_and_where_clause(
                type_params_pair,
                where_clause_pair,
                config,
            )
            .unwrap_or_else(&mut warnings, &mut errors, || Vec::new());
        ok(
            TraitDeclaration {
                type_parameters,
                name,
                interface_surface: interface,
                methods,
                visibility,
            },
            warnings,
            errors,
        )
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct TraitFn<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) parameters: Vec<FunctionParameter<'sc>>,
    pub(crate) return_type: TypeInfo,
    pub(crate) return_type_span: Span<'sc>,
}

impl<'sc> TraitFn<'sc> {
    pub(crate) fn parse_from_pair(
        pair: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        let path = config.map(|c| c.path());
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut signature = pair.clone().into_inner();
        let whole_fn_sig_span = Span {
            span: pair.as_span(),
            path: path.clone(),
        };
        let _fn_keyword = signature.next().unwrap();
        let name = signature.next().unwrap();
        let name_span = Span {
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
            is_snake_case(name.primary_name),
            warnings,
            name_span,
            Warning::NonSnakeCaseFunctionName {
                name: name.primary_name
            }
        );
        let parameters = signature.next().unwrap();
        let parameters = check!(
            FunctionParameter::list_from_pairs(parameters.into_inner(), config),
            Vec::new(),
            warnings,
            errors
        );
        let return_type_signal = signature.next();
        let (return_type, return_type_span) = match return_type_signal {
            Some(_) => {
                let pair = signature.next().unwrap();
                let span = Span {
                    span: pair.as_span(),
                    path: path.clone(),
                };
                (
                    check!(
                        TypeInfo::parse_from_pair(pair, config),
                        TypeInfo::ErrorRecovery,
                        warnings,
                        errors
                    ),
                    span,
                )
            }
            None => (TypeInfo::Unit, whole_fn_sig_span),
        };

        ok(
            TraitFn {
                name,
                parameters,
                return_type,
                return_type_span,
            },
            warnings,
            errors,
        )
    }
}
