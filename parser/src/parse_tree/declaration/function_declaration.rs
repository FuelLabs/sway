use crate::error::{ParseResult, Warning};
use crate::parse_tree::{declaration::TypeParameter, Expression, VarName};
use crate::types::TypeInfo;
use crate::{CodeBlock, ParseError, Rule};
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
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> ParseResult<'sc, Self> {
        let mut parts = pair.clone().into_inner();
        let mut warnings = Vec::new();
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

        let parameters = FunctionParameter::list_from_pairs(parameters_pair.into_inner())?;
        let return_type = match return_type_pair {
            Some(pair) => TypeInfo::parse_from_pair(pair)?,
            None => TypeInfo::Unit,
        };
        let type_parameters = TypeParameter::parse_from_type_params_and_where_clause(
            type_params_pair,
            where_clause_pair,
        )?;
        let body = parts.next().unwrap();
        let body = eval!(CodeBlock::parse_from_pair, warnings, body);
        Ok((
            FunctionDeclaration {
                name,
                parameters,
                body,
                span: pair.as_span(),
                return_type,
                type_parameters,
            },
            warnings,
        ))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionParameter<'sc> {
    pub(crate) name: &'sc str,
    pub(crate) r#type: TypeInfo<'sc>,
}

impl<'sc> FunctionParameter<'sc> {
    pub(crate) fn list_from_pairs(
        pairs: impl Iterator<Item = Pair<'sc, Rule>>,
    ) -> Result<Vec<FunctionParameter<'sc>>, ParseError<'sc>> {
        pairs
            .map(|pair: Pair<'sc, Rule>| {
                let mut parts = pair.clone().into_inner();
                let name = parts.next().unwrap().as_str();
                let r#type = if name == "self" {
                    TypeInfo::SelfType
                } else {
                    TypeInfo::parse_from_pair_inner(parts.next().unwrap())?
                };
                Ok(FunctionParameter { name, r#type })
            })
            .collect()
    }
}
