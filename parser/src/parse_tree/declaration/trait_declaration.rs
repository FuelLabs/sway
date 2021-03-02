use super::{FunctionDeclaration, FunctionParameter};
use crate::error::*;
use crate::parse_tree::VarName;
use crate::parser::{HllParser, Rule};
use crate::types::TypeInfo;
use either::*;
use inflector::cases::snakecase::is_snake_case;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub(crate) struct TraitDeclaration<'sc> {
    pub(crate) name: VarName<'sc>,
    pub(crate) interface_surface: Vec<TraitFn<'sc>>,
    pub(crate) methods: Vec<FunctionDeclaration<'sc>>,
}

impl<'sc> TraitDeclaration<'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut trait_parts = pair.into_inner();
        let _trait_keyword = trait_parts.next();
        let name_pair = trait_parts.next().unwrap();
        let name = VarName {
            primary_name: name_pair.as_str(),
            sub_names: vec![],
            span: name_pair.as_span(),
        };
        let mut methods = Vec::new();
        let mut interface = Vec::new();

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
                    _ => unreachable!(),
                }
            }
        }
        ok(
            TraitDeclaration {
                name,
                interface_surface: interface,
                methods,
            },
            warnings,
            errors,
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TraitFn<'sc> {
    pub(crate) name: &'sc str,
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
        let mut name_span = name.as_span();
        let name = name.as_str();
        assert_or_warn!(
            is_snake_case(name),
            warnings,
            name_span,
            Warning::NonSnakeCaseFunctionName { name }
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
