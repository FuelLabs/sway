use crate::{
    build_config::BuildConfig,
    error::*,
    parse_tree::{declaration::TypeParameter, ident, Visibility},
    style::{is_snake_case, is_upper_camel_case},
    type_engine::{insert_type, look_up_type_id, TypeId, TypeInfo},
    CodeBlock, Rule,
};

use sway_types::{ident::Ident, span::Span, Function, Property};

use pest::iterators::Pair;

mod purity;
pub use purity::Purity;

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub purity: Purity,
    pub name: Ident,
    pub visibility: Visibility,
    pub body: CodeBlock,
    pub parameters: Vec<FunctionParameter>,
    pub span: Span,
    pub return_type: TypeInfo,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub(crate) return_type_span: Span,
}

impl FunctionDeclaration {
    pub fn parse_from_pair(pair: Pair<Rule>, config: Option<&BuildConfig>) -> CompileResult<Self> {
        let path = config.map(|c| c.path());
        let mut parts = pair.clone().into_inner();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let signature_or_visibility = parts.next().unwrap();
        let (visibility, signature) = if signature_or_visibility.as_rule() == Rule::visibility {
            (
                Visibility::parse_from_pair(signature_or_visibility),
                parts.next().unwrap().into_inner(),
            )
        } else {
            (Visibility::Private, signature_or_visibility.into_inner())
        };
        let mut signature = signature.peekable();
        let purity = if signature
            .peek()
            .map(|x| x.as_rule() == Rule::impurity_keyword)
            .unwrap_or(false)
        {
            let _ = signature.next();
            Purity::Impure
        } else {
            Purity::Pure
        };
        let _fn_keyword = signature.next().unwrap();
        let name = check!(
            ident::parse_from_pair(signature.next().unwrap(), config),
            return err(warnings, errors),
            warnings,
            errors
        );
        assert_or_warn!(
            is_snake_case(name.as_str()),
            warnings,
            name.span().clone(),
            Warning::NonSnakeCaseFunctionName { name: name.clone() }
        );

        let mut type_params_pair = None;
        let mut where_clause_pair = None;
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
                    where_clause_pair = Some(pair);
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

        let type_parameters = check!(
            TypeParameter::parse_from_type_params_and_where_clause(
                type_params_pair,
                where_clause_pair,
                config,
            ),
            vec!(),
            warnings,
            errors
        );
        for type_parameter in type_parameters.iter() {
            assert_or_warn!(
                is_upper_camel_case(type_parameter.name_ident.as_str()),
                warnings,
                type_parameter.name_ident.span().clone(),
                Warning::NonClassCaseTypeParameter {
                    name: type_parameter.name_ident.clone()
                }
            );
        }

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

        let body = parts.next().unwrap();
        let whole_block_span = Span::from_pest(body.as_span(), path.clone());
        let body = check!(
            CodeBlock::parse_from_pair(body, config),
            crate::CodeBlock {
                contents: Vec::new(),
                whole_block_span,
            },
            warnings,
            errors
        );

        ok(
            FunctionDeclaration {
                purity,
                name,
                parameters,
                return_type_span,
                visibility,
                body,
                span: Span::from_pest(pair.as_span(), path),
                return_type,
                type_parameters,
            },
            warnings,
            errors,
        )
    }

    pub fn parse_json_abi(&self) -> Function {
        Function {
            name: self.name.as_str().to_string(),
            type_field: "function".to_string(),
            inputs: self
                .parameters
                .iter()
                .map(|x| Property {
                    name: x.name.as_str().to_string(),
                    type_field: look_up_type_id(x.type_id).friendly_type_str(),
                    components: None,
                })
                .collect(),
            outputs: vec![Property {
                name: "".to_string(),
                type_field: self.return_type.friendly_type_str(),
                components: None,
            }],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionParameter {
    pub name: Ident,
    pub(crate) type_id: TypeId,
    pub(crate) type_span: Span,
}

impl FunctionParameter {
    pub(crate) fn list_from_pairs(
        pairs: impl Iterator<Item = Pair<Rule>>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Vec<FunctionParameter>> {
        let path = config.map(|c| c.path());
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut pairs_buf = Vec::new();
        for pair in pairs {
            if pair.as_str().trim() == "self" {
                let type_span = Span::from_pest(pair.as_span(), path.clone());
                let type_id = insert_type(TypeInfo::SelfType);
                let name =
                    Ident::new_with_override("self", Span::from_pest(pair.as_span(), path.clone()));
                pairs_buf.push(FunctionParameter {
                    name,
                    type_id,
                    type_span,
                });
                continue;
            }
            let mut parts = pair.clone().into_inner();
            let name_pair = parts.next().unwrap();
            let name = check!(
                ident::parse_from_pair(name_pair, config),
                return err(warnings, errors),
                warnings,
                errors
            );
            let type_pair = parts.next().unwrap();
            let type_span = Span::from_pest(type_pair.as_span(), path.clone());
            let type_id = insert_type(check!(
                TypeInfo::parse_from_pair(type_pair, config),
                TypeInfo::ErrorRecovery,
                warnings,
                errors
            ));
            pairs_buf.push(FunctionParameter {
                name,
                type_id,
                type_span,
            });
        }
        ok(pairs_buf, warnings, errors)
    }
}
