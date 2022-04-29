use super::{FunctionDeclaration, FunctionParameter};

use crate::{
    build_config::BuildConfig,
    error::*,
    parse_tree::{ident, CallPath, Visibility},
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
    pub methods: Vec<FunctionDeclaration>,
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

        let name = check!(
            ident::parse_from_pair(trait_parts.next().unwrap(), config),
            return err(warnings, errors),
            warnings,
            errors
        );
        assert_or_warn!(
            is_upper_camel_case(name.as_str()),
            warnings,
            name.span().clone(),
            Warning::NonClassCaseTraitName { name: name.clone() }
        );

        let supertraits = match trait_parts.peek() {
            Some(pair) => match pair.as_rule() {
                Rule::supertraits => {
                    let mut supertraits = vec![];
                    for x in trait_parts.next().unwrap().into_inner() {
                        supertraits.push(check!(
                            Supertrait::parse_from_pair(x, config),
                            continue,
                            warnings,
                            errors
                        ));
                    }
                    supertraits
                }
                _ => vec![],
            },
            None => vec![],
        };

        let mut interface = Vec::new();
        let mut methods = Vec::new();
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

        ok(
            TraitDeclaration {
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct Supertrait {
    pub(crate) name: CallPath,
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
        ok(Supertrait { name }, warnings, errors)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TraitFn {
    pub name: Ident,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: TypeInfo,
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
        let name_span = Span::from_pest(name.as_span(), path.clone());
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
                        Span::from_pest(pair.as_span(), path.clone()),
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
        let pest_span = if let Some(ref pair) = return_type_pair {
            pair.as_span()
        } else {
            /* if this has no return type, just use the fn params as the span. */
            parameters_span
        };
        let return_type_span = Span::from_pest(pest_span, path.clone());
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
                Span::from_pest(type_params.as_span(), path),
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
