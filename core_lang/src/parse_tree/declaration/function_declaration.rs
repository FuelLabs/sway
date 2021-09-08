use crate::build_config::BuildConfig;
use crate::error::*;
use crate::parse_tree::declaration::TypeParameter;
use crate::span::Span;
use crate::types::TypeInfo;
use crate::{CodeBlock, Ident, Rule};
use inflector::cases::snakecase::is_snake_case;
use pest::iterators::Pair;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Visibility {
    Public,
    Private,
}

impl Visibility {
    pub(crate) fn parse_from_pair<'sc>(input: Pair<'sc, Rule>) -> Self {
        match input.as_str().trim() {
            "pub" => Visibility::Public,
            _ => Visibility::Private,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionDeclaration<'sc> {
    pub name: Ident<'sc>,
    pub(crate) visibility: Visibility,
    pub body: CodeBlock<'sc>,
    pub(crate) parameters: Vec<FunctionParameter<'sc>>,
    pub(crate) span: Span<'sc>,
    pub(crate) return_type: TypeInfo<'sc>,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
    pub(crate) return_type_span: Span<'sc>,
}

impl<'sc> FunctionDeclaration<'sc> {
    pub fn parse_from_pair(
        pair: Pair<'sc, Rule>,
        config: Option<BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        let path = config.clone().map(|c| c.dir_of_code);
        let mut parts = pair.clone().into_inner();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let signature_or_visibility = parts.next().unwrap();
        let (visibility, mut signature) = if signature_or_visibility.as_rule() == Rule::visibility {
            (
                Visibility::parse_from_pair(signature_or_visibility),
                parts.next().unwrap().into_inner(),
            )
        } else {
            (Visibility::Private, signature_or_visibility.into_inner())
        };
        let _fn_keyword = signature.next().unwrap();
        let name = signature.next().unwrap();
        let name_span = Span {
            span: name.as_span(),
            path: path.clone(),
        };
        let name = eval2!(
            Ident::parse_from_pair,
            warnings,
            errors,
            name,
            config.clone(),
            return err(warnings, errors)
        );
        assert_or_warn!(
            is_snake_case(name.primary_name),
            warnings,
            name_span,
            Warning::NonSnakeCaseFunctionName {
                name: name.primary_name.to_string()
            }
        );
        let mut type_params_pair = None;
        let mut where_clause_pair = None;
        let mut parameters_pair = None;
        let mut return_type_pair = None;
        while let Some(pair) = signature.next() {
            match pair.as_rule() {
                Rule::type_params => {
                    type_params_pair = Some(pair);
                }
                Rule::return_type => {
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

        let parameters = eval2!(
            FunctionParameter::list_from_pairs,
            warnings,
            errors,
            parameters_pair.clone().into_inner(),
            config.clone(),
            Vec::new()
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
            Some(ref pair) => eval2!(
                TypeInfo::parse_from_pair,
                warnings,
                errors,
                pair,
                config.clone(),
                TypeInfo::Unit
            ),
            None => TypeInfo::Unit,
        };
        let type_parameters = TypeParameter::parse_from_type_params_and_where_clause(
            type_params_pair,
            where_clause_pair,
            config.clone(),
        )
        .unwrap_or_else(&mut warnings, &mut errors, || Vec::new());

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
        let body = eval2!(
            CodeBlock::parse_from_pair,
            warnings,
            errors,
            body,
            config.clone(),
            crate::CodeBlock {
                contents: Vec::new(),
                whole_block_span,
                scope: Default::default()
            }
        );
        ok(
            FunctionDeclaration {
                name,
                parameters,
                return_type_span,
                visibility,
                body,
                span: Span {
                    span: pair.as_span(),
                    path: path.clone(),
                },
                return_type,
                type_parameters,
            },
            warnings,
            errors,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FunctionParameter<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) r#type: TypeInfo<'sc>,
    pub(crate) type_span: Span<'sc>,
}

impl<'sc> FunctionParameter<'sc> {
    pub(crate) fn list_from_pairs(
        pairs: impl Iterator<Item = Pair<'sc, Rule>>,
        config: Option<BuildConfig>,
    ) -> CompileResult<'sc, Vec<FunctionParameter<'sc>>> {
        let path = config.clone().map(|c| c.dir_of_code);
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
                let name = Ident {
                    span: Span {
                        span: pair.as_span(),
                        path: path.clone(),
                    },
                    primary_name: "self",
                };
                pairs_buf.push(FunctionParameter {
                    name,
                    r#type,
                    type_span,
                });
                continue;
            }
            let mut parts = pair.clone().into_inner();
            let name_pair = parts.next().unwrap();
            let name = eval2!(
                Ident::parse_from_pair,
                warnings,
                errors,
                name_pair,
                config.clone(),
                return err(warnings, errors)
            );
            let type_pair = parts.next().unwrap();
            let type_span = Span {
                span: type_pair.as_span(),
                path: path.clone(),
            };
            let r#type = eval2!(
                TypeInfo::parse_from_pair_inner,
                warnings,
                errors,
                type_pair,
                config.clone(),
                TypeInfo::ErrorRecovery
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
