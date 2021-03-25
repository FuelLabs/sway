use super::{FunctionDeclaration, FunctionParameter};
use crate::error::*;
use crate::parse_tree::{Ident, TypeParameter};
use crate::parser::Rule;
use crate::types::TypeInfo;
use inflector::cases::classcase::is_class_case;
use inflector::cases::snakecase::is_snake_case;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub(crate) struct TraitDeclaration<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) interface_surface: Vec<TraitFn<'sc>>,
    pub(crate) methods: Vec<FunctionDeclaration<'sc>>,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
}

impl<'sc> TraitDeclaration<'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut trait_parts = pair.into_inner().peekable();
        let _trait_keyword = trait_parts.next();
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
            is_class_case(name_pair.as_str()),
            warnings,
            span,
            Warning::NonClassCaseTraitName {
                name: name_pair.as_str()
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
        let type_parameters = match crate::parse_tree::declaration::TypeParameter::parse_from_type_params_and_where_clause(
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
        ok(
            TraitDeclaration {
                type_parameters,
                name,
                interface_surface: interface,
                methods,
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
}

impl<'sc> TraitFn<'sc> {
    fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut signature = pair.clone().into_inner();
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
        let return_type = match return_type_signal {
            Some(_) => eval!(
                TypeInfo::parse_from_pair,
                warnings,
                errors,
                signature.next().unwrap(),
                TypeInfo::ErrorRecovery
            ),
            None => TypeInfo::Unit,
        };

        ok(
            TraitFn {
                name,
                parameters,
                return_type,
            },
            warnings,
            errors,
        )
    }
}
