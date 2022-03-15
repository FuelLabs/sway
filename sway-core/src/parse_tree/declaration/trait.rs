use super::{FunctionDeclaration, FunctionParameter};

use crate::{
    build_config::BuildConfig,
    error::*,
    parse_tree::{ident, CallPath, TypeParameter, Visibility},
    parser::Rule,
    style::{is_snake_case, is_upper_camel_case},
    type_engine::TypeInfo,
};

use sway_types::{ident::Ident, span::Span};

use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct TraitDeclaration {
    pub name: Ident,
    pub(crate) interface_surface: Vec<TraitFn>,
    pub(crate) methods: Vec<FunctionDeclaration>,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub(crate) supertraits: Vec<Supertrait>,
    pub visibility: Visibility,
}

impl TraitDeclaration {
    pub(crate) fn parse_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Self> {
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
            ident::parse_from_pair(name_pair.clone(), config),
            return err(warnings, errors),
            warnings,
            errors
        );
        let span = name.span().clone();
        assert_or_warn!(
            is_upper_camel_case(name_pair.as_str().trim()),
            warnings,
            span,
            Warning::NonClassCaseTraitName { name: name.clone() }
        );
        let mut type_params_pair = None;
        let mut where_clause_pair = None;
        let mut methods = Vec::new();
        let mut interface = Vec::new();
        let mut supertraits = Vec::new();

        for _ in 0..3 {
            match trait_parts.peek().map(|x| x.as_rule()) {
                Some(Rule::supertraits) => {
                    for supertrait in trait_parts.next().unwrap().into_inner() {
                        supertraits.push(check!(
                            Supertrait::parse_from_pair(supertrait, config),
                            continue,
                            warnings,
                            errors
                        ));
                    }
                }
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
        let type_parameters = TypeParameter::parse_from_type_params_and_where_clause(
            type_params_pair,
            where_clause_pair,
            config,
        )
        .unwrap_or_else(&mut warnings, &mut errors, Vec::new);
        ok(
            TraitDeclaration {
                type_parameters,
                name,
                interface_surface: interface,
                methods,
                supertraits,
                visibility,
            },
            warnings,
            errors,
        )
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Supertrait {
    pub(crate) name: CallPath,
    pub(crate) type_parameters: Vec<TypeParameter>,
}

impl Supertrait {
    pub(crate) fn parse_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut supertrait_parts = pair.into_inner();
        let name = supertrait_parts.next().unwrap();
        let name = check!(
            CallPath::parse_from_pair(name, config),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut type_params_pair = None;

        if let Some(type_params) = supertrait_parts.next() {
            match type_params.as_rule() {
                Rule::type_params => {
                    type_params_pair = Some(type_params);
                }
                _ => unreachable!(),
            }
        }

        let type_parameters =
            TypeParameter::parse_from_type_params_and_where_clause(type_params_pair, None, config)
                .unwrap_or_else(&mut warnings, &mut errors, Vec::new);
        ok(
            Supertrait {
                name,
                type_parameters,
            },
            warnings,
            errors,
        )
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct TraitFn {
    pub(crate) name: Ident,
    pub(crate) parameters: Vec<FunctionParameter>,
    pub(crate) return_type: TypeInfo,
    pub(crate) return_type_span: Span,
}

impl TraitFn {
    pub(crate) fn parse_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Self> {
        let path = config.map(|c| c.path());
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut signature = pair.into_inner();
        let _fn_keyword = signature.next().unwrap();
        let name = signature.next().unwrap();
        let name_span = Span {
            span: name.as_span(),
            path: path.clone(),
        };
        let name = check!(
            ident::parse_from_pair(name, config),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut type_params_pair = None;
        let mut _where_clause_pair = None;
        let mut parameters_pair = None;
        let mut return_type_pair = None;
        for pair in signature {
            match pair.as_rule() {
                Rule::type_params => {
                    type_params_pair = Some(pair);
                }
                Rule::type_name => {
                    return_type_pair = Some(pair);
                }
                Rule::fn_decl_params => {
                    parameters_pair = Some(pair);
                }
                Rule::trait_bounds => {
                    _where_clause_pair = Some(pair);
                }
                Rule::fn_returns => (),
                _ => {
                    errors.push(CompileError::Internal(
                        "Unexpected token while parsing function signature.",
                        Span {
                            span: pair.as_span(),
                            path: path.clone(),
                        },
                    ));
                }
            }
        }
        assert_or_warn!(
            is_snake_case(name.as_str()),
            warnings,
            name_span,
            Warning::NonSnakeCaseFunctionName { name: name.clone() }
        );
        // these are non-optional in a func decl
        let parameters_pair = parameters_pair.unwrap();
        let parameters_span = parameters_pair.as_span();

        let parameters = check!(
            FunctionParameter::list_from_pairs(parameters_pair.into_inner(), config),
            Vec::new(),
            warnings,
            errors
        );
        let return_type_span = Span {
            span: if let Some(ref pair) = return_type_pair {
                pair.as_span()
            } else {
                /* if this has no return type, just use the fn params as the span. */
                parameters_span
            },
            path: path.clone(),
        };
        let return_type = match return_type_pair {
            Some(ref pair) => check!(
                TypeInfo::parse_from_pair(pair.clone(), config),
                TypeInfo::Tuple(Vec::new()),
                warnings,
                errors
            ),
            None => TypeInfo::Tuple(Vec::new()),
        };
        if let Some(type_params) = type_params_pair {
            errors.push(CompileError::Unimplemented(
                "Generic traits have not yet been implemented.",
                Span {
                    span: type_params.as_span(),
                    path,
                },
            ));
        }

        /* when we implement generic traits, uncomment this to get the type params and where
                 * clause
                let type_parameters = TypeParameter::parse_from_type_params_and_where_clause(
                    type_params_pair,
                    where_clause_pair,
                    config,
                )
                .unwrap_or_else(&mut warnings, &mut errors, Vec::new);
        */

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
