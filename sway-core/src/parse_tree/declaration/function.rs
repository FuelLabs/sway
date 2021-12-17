use crate::build_config::BuildConfig;
use crate::error::*;
use crate::parse_tree::{declaration::TypeParameter, Visibility};
use crate::span::Span;
use crate::style::is_snake_case;
use crate::type_engine::TypeInfo;
use crate::{CodeBlock, Ident, Rule};
use pest::iterators::Pair;
use sway_types::{Function, Property};

mod purity;
pub use purity::Purity;

#[derive(Debug, Clone)]
pub struct FunctionDeclaration<'sc> {
    pub purity: Purity,
    pub name: Ident<'sc>,
    pub visibility: Visibility,
    pub body: CodeBlock<'sc>,
    pub(crate) parameters: Vec<FunctionParameter<'sc>>,
    pub span: Span<'sc>,
    pub(crate) return_type: TypeInfo,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
    pub(crate) return_type_span: Span<'sc>,
}

impl<'sc> FunctionDeclaration<'sc> {
    pub fn parse_from_pair(
        pair: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Self> {
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
            is_snake_case(name.primary_name()),
            warnings,
            name_span,
            Warning::NonSnakeCaseFunctionName {
                name: name.primary_name()
            }
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
                        Span {
                            span: pair.as_span(),
                            path: path.clone(),
                        },
                    ));
                }
            }
        }

        // these are non-optional in a func decl
        let parameters_pair = parameters_pair.unwrap();
        let parameters_span = parameters_pair.as_span();

        let parameters = check!(
            FunctionParameter::list_from_pairs(parameters_pair.clone().into_inner(), config),
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
        let type_parameters = TypeParameter::parse_from_type_params_and_where_clause(
            type_params_pair,
            where_clause_pair,
            config,
        )
        .unwrap_or_else(&mut warnings, &mut errors, Vec::new);

        // check that all generic types used in function parameters are a part of the type
        // parameters
        /*
        let mut generic_params_buf_for_error_message = Vec::new();
        for param in parameters.iter() {
            if let TypeInfo::Generic { name } = param.r#type {
                generic_params_buf_for_error_message.push(name);
            }
        }
        let comma_separated_generic_params = generic_params_buf_for_error_message.join(", ");
        for param in parameters.iter() {
            if let TypeInfo::Generic { name: st } = param.r#type {
                if type_parameters
                    .iter()
                    .find(|TypeParameter { name, .. }| *name == st)
                    .is_none()
                {
                    errors.push(CompileError::TypeParameterNotInTypeScope {
                        name: st,
                        span: param.name.span.clone(),
                        comma_separated_generic_params: comma_separated_generic_params.clone(),
                        fn_name: name,
                        args: parameters_pair.clone().as_str(),
                        return_type: return_type_pair
                            .clone()
                            .map(|x| x.as_str().to_string())
                            .unwrap_or(TypeInfo::Unit.friendly_type_str()),
                    });
                }
            }
        }
        */
        let body = parts.next().unwrap();
        let whole_block_span = Span {
            span: body.as_span(),
            path: path.clone(),
        };
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
                span: Span {
                    span: pair.as_span(),
                    path,
                },
                return_type,
                type_parameters,
            },
            warnings,
            errors,
        )
    }

    pub fn parse_json_abi(&self) -> Function {
        Function {
            name: self.name.primary_name().to_string(),
            type_field: "function".to_string(),
            inputs: self
                .parameters
                .iter()
                .map(|x| Property {
                    name: x.name.primary_name().to_string(),
                    type_field: x.r#type.friendly_type_str(),
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
pub(crate) struct FunctionParameter<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) r#type: TypeInfo,
    pub(crate) type_span: Span<'sc>,
}

impl<'sc> FunctionParameter<'sc> {
    pub(crate) fn list_from_pairs(
        pairs: impl Iterator<Item = Pair<'sc, Rule>>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Vec<FunctionParameter<'sc>>> {
        let path = config.map(|c| c.path());
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut pairs_buf = Vec::new();
        for pair in pairs {
            if pair.as_str().trim() == "self" {
                let type_span = Span {
                    span: pair.as_span(),
                    path: path.clone(),
                };
                let r#type = TypeInfo::SelfType;
                let name = Ident::new_with_override(
                    "self",
                    Span {
                        span: pair.as_span(),
                        path: path.clone(),
                    },
                );
                pairs_buf.push(FunctionParameter {
                    name,
                    r#type,
                    type_span,
                });
                continue;
            }
            let mut parts = pair.clone().into_inner();
            let name_pair = parts.next().unwrap();
            let name = check!(
                Ident::parse_from_pair(name_pair, config),
                return err(warnings, errors),
                warnings,
                errors
            );
            let type_pair = parts.next().unwrap();
            let type_span = Span {
                span: type_pair.as_span(),
                path: path.clone(),
            };
            let r#type = check!(
                TypeInfo::parse_from_pair(type_pair, config),
                TypeInfo::ErrorRecovery,
                warnings,
                errors
            );
            pairs_buf.push(FunctionParameter {
                name,
                r#type,
                type_span,
            });
        }
        ok(pairs_buf, warnings, errors)
    }
}
