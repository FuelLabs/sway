use super::{FunctionDeclaration, FunctionParameter, Visibility};
use crate::parse_tree::TypeParameter;
use crate::parser::Rule;
use crate::types::TypeInfo;
use crate::{error::*, Ident};
use inflector::cases::classcase::is_class_case;
use inflector::cases::snakecase::is_snake_case;
use pest::iterators::Pair;
use pest::Span;

#[derive(Debug, Clone)]
pub struct TraitDeclaration<'sc> {
    pub name: Ident<'sc>,
    pub(crate) interface_surface: Vec<TraitFn<'sc>>,
    pub(crate) methods: Vec<FunctionDeclaration<'sc>>,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
    pub(crate) visibility: Visibility,
}

impl<'sc> TraitDeclaration<'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
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
        let name = eval!(
            Ident::parse_from_pair,
            warnings,
            errors,
            name_pair,
            return err(warnings, errors)
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
                        interface.push(eval!(
                            TraitFn::parse_from_pair,
                            warnings,
                            errors,
                            fn_sig_or_decl,
                            continue
                        ));
                    }
                    Rule::fn_decl => {
                        methods.push(eval!(
                            FunctionDeclaration::parse_from_pair,
                            warnings,
                            errors,
                            fn_sig_or_decl,
                            continue
                        ));
                    }
                    a => unreachable!("{:?}", a),
                }
            }
        }
        let mut parsed_type_parameters =
            crate::parse_tree::declaration::TypeParameter::parse_from_type_params_and_where_clause(
                type_params_pair,
                where_clause_pair,
            );
        warnings.append(&mut parsed_type_parameters.warnings);
        errors.append(&mut parsed_type_parameters.errors);
        let type_parameters = parsed_type_parameters.value.unwrap_or_else(|| Vec::new());
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
    pub(crate) return_type: TypeInfo<'sc>,
    pub(crate) return_type_span: Span<'sc>,
}

impl<'sc> TraitFn<'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut signature = pair.clone().into_inner();
        let whole_fn_sig_span = pair.as_span();
        let _fn_keyword = signature.next().unwrap();
        let name = signature.next().unwrap();
        let name_span = name.as_span();
        let name = eval!(
            Ident::parse_from_pair,
            warnings,
            errors,
            name,
            return err(warnings, errors)
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
        let parameters = eval!(
            FunctionParameter::list_from_pairs,
            warnings,
            errors,
            parameters.into_inner(),
            Vec::new()
        );
        let return_type_signal = signature.next();
        let (return_type, return_type_span) = match return_type_signal {
            Some(_) => {
                let pair = signature.next().unwrap();
                let span = pair.as_span();
                (
                    eval!(
                        TypeInfo::parse_from_pair,
                        warnings,
                        errors,
                        pair,
                        TypeInfo::ErrorRecovery
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
