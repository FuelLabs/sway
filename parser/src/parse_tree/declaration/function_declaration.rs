use crate::error::*;
use crate::parse_tree::{declaration::TypeParameter, Expression, VarName};
use crate::types::TypeInfo;
use crate::{CodeBlock, CompileError, Rule};
use either::Either;
use inflector::cases::snakecase::is_snake_case;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub(crate) struct FunctionDeclaration<'sc> {
    pub(crate) name: &'sc str,
    pub(crate) body: CodeBlock<'sc>,
    pub(crate) parameters: Vec<FunctionParameter<'sc>>,
    pub(crate) span: pest::Span<'sc>,
    pub(crate) return_type: TypeInfo<'sc>,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
}

impl<'sc> FunctionDeclaration<'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let mut parts = pair.clone().into_inner();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut signature = parts.next().unwrap().into_inner();
        let _fn_keyword = signature.next().unwrap();
        let name = signature.next().unwrap();
        let name_span = name.as_span();
        let name = name.as_str();
        assert_or_warn!(
            is_snake_case(name),
            warnings,
            name_span,
            Warning::NonSnakeCaseFunctionName { name }
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

        let parameters = eval!(FunctionParameter::list_from_pairs, warnings, errors, parameters_pair.into_inner(), Vec::new());
        let return_type = match return_type_pair {
            Some(pair) => eval!(TypeInfo::parse_from_pair, warnings, errors, pair, TypeInfo::Unit ),
            None => TypeInfo::Unit,
        };
        let type_parameters = match TypeParameter::parse_from_type_params_and_where_clause(
            type_params_pair,
            where_clause_pair,
        ) {
            CompileResult::Ok{ value, warnings: mut l_w} => {
                        warnings.append(&mut l_w);
                        value
            },
            CompileResult::Err{ warnings: mut l_w, errors: mut l_e} => {
                warnings.append(&mut l_w);
                errors.append(&mut l_e);
                Vec::new()
            }
        };
        let body = parts.next().unwrap();
        let body = eval!(CodeBlock::parse_from_pair, warnings, errors, body, crate::CodeBlock { contents: Vec::new(), scope: Default::default()});
        ok(
            FunctionDeclaration {
                name,
                parameters,
                body,
                span: pair.as_span(),
                return_type,
                type_parameters,
            },
            warnings,
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionParameter<'sc> {
    pub(crate) name: VarName<'sc>,
    pub(crate) r#type: TypeInfo<'sc>,
}

impl<'sc> FunctionParameter<'sc> {
    pub(crate) fn list_from_pairs(
        pairs: impl Iterator<Item = Pair<'sc, Rule>>,
    ) -> CompileResult<'sc,Vec<FunctionParameter<'sc>>> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut pairs_buf = Vec::new();
        for pair in pairs {
                let mut parts = pair.clone().into_inner();
                let name_pair = parts.next().unwrap();
                let name_str = name_pair.as_str();
                let name = eval!(VarName::parse_from_pair, warnings, errors, name_pair, VarName {
                    primary_name: "error parsing var name",
                    sub_names: Vec::new(),
                    span: name_pair.as_span()
                });
                let r#type = if name_str == "self" {
                    TypeInfo::SelfType
                } else {
                    eval!(TypeInfo::parse_from_pair_inner, warnings, errors, parts.next().unwrap(), TypeInfo::ErrorRecovery)
                };
                pairs_buf.push(FunctionParameter { name, r#type });
            }
            ok(pairs_buf, warnings)
    }
}
