use crate::error::*;
use crate::parse_tree::{declaration::TypeParameter, Expression, VarName};
use crate::types::TypeInfo;
use crate::{CodeBlock, CompileError, Rule};
use either::Either;
use inflector::cases::snakecase::is_snake_case;
use pest::iterators::Pair;
use pest::Span;

#[derive(Debug, Clone)]
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
pub(crate) struct FunctionDeclaration<'sc> {
    pub(crate) name: VarName<'sc>,
    pub(crate) visibility: Visibility,
    pub(crate) body: CodeBlock<'sc>,
    pub(crate) parameters: Vec<FunctionParameter<'sc>>,
    pub(crate) span: pest::Span<'sc>,
    pub(crate) return_type: TypeInfo<'sc>,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
    pub(crate) return_type_span: Span<'sc>,
}

impl<'sc> FunctionDeclaration<'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let mut parts = pair.clone().into_inner();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut signature_or_visibility = parts.next().unwrap();
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
        let name_span = name.as_span();
        let name = eval!(
            VarName::parse_from_pair,
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
                a => todo!("What is this? {:?}", a),
            }
        }

        // these are non-optional in a func decl
        let parameters_pair = parameters_pair.unwrap();
        let parameters_span = parameters_pair.as_span();

        let parameters = eval!(
            FunctionParameter::list_from_pairs,
            warnings,
            errors,
            parameters_pair.clone().into_inner(),
            Vec::new()
        );
        let return_type_span = if let Some(ref pair) = return_type_pair {
            pair.as_span()
        } else {
            /* if this has no return type, just use the fn params as the span. */
            parameters_span
        };
        let return_type = match return_type_pair {
            Some(ref pair) => eval!(
                TypeInfo::parse_from_pair,
                warnings,
                errors,
                pair,
                TypeInfo::Unit
            ),
            None => TypeInfo::Unit,
        };
        let type_parameters = match TypeParameter::parse_from_type_params_and_where_clause(
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
        let body = eval!(
            CodeBlock::parse_from_pair,
            warnings,
            errors,
            body,
            crate::CodeBlock {
                contents: Vec::new(),
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
                span: pair.as_span(),
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
    pub(crate) name: VarName<'sc>,
    pub(crate) r#type: TypeInfo<'sc>,
    pub(crate) type_span: Span<'sc>,
}

impl<'sc> FunctionParameter<'sc> {
    pub(crate) fn list_from_pairs(
        pairs: impl Iterator<Item = Pair<'sc, Rule>>,
    ) -> CompileResult<'sc, Vec<FunctionParameter<'sc>>> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut pairs_buf = Vec::new();
        for pair in pairs {
            if pair.as_str().trim() == "self" {
                let type_span = pair.as_span();
                let r#type = TypeInfo::SelfType;
                let name = VarName {
                    span: pair.as_span(),
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
            let name_str = name_pair.as_str();
            let name = eval!(
                VarName::parse_from_pair,
                warnings,
                errors,
                name_pair,
                VarName {
                    primary_name: "error parsing var name",
                    span: name_pair.as_span()
                }
            );
            let type_pair = parts.next().unwrap();
            let type_span = type_pair.as_span();
            let r#type = eval!(
                TypeInfo::parse_from_pair_inner,
                warnings,
                errors,
                type_pair,
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
