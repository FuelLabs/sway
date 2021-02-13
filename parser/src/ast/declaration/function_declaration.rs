use crate::ast::Expression;
use crate::{CodeBlock, CompileError, Rule};
use either::Either;
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

#[derive(Debug, Clone)]
pub(crate) struct TypeParameter<'sc> {
    name: &'sc str,
    trait_constraint: Vec<TraitConstraint<'sc>>,
}

impl<'sc> TypeParameter<'sc> {
    fn parse_from_type_params_and_where_clause(
        type_params_pair: Option<Pair<'sc, Rule>>,
        where_clause_pair: Option<Pair<'sc, Rule>>,
    ) -> Result<Vec<TypeParameter<'sc>>, CompileError<'sc>> {
        let mut params: Vec<TypeParameter> = match type_params_pair {
            Some(type_params_pair) => type_params_pair
                .into_inner()
                .map(|pair| TypeParameter {
                    name: pair.as_str(),
                    trait_constraint: Vec::new(),
                })
                .collect(),
            None => {
                // no type params specified, ensure where clause is empty
                if let Some(where_clause_pair) = where_clause_pair {
                    return Err(CompileError::UnexpectedWhereClause(
                        where_clause_pair.as_span(),
                    ));
                }
                Vec::new()
            }
        };
        match where_clause_pair {
            Some(where_clause_pair) => {
                let mut pair = where_clause_pair.into_inner().peekable();
                while pair.peek().is_some() {
                    let type_param = pair.next().unwrap();
                    assert_eq!(type_param.as_rule(), Rule::generic_type_param);
                    let trait_constraint = pair.next().unwrap();
                    assert_eq!(trait_constraint.as_rule(), Rule::trait_name);
                    // assign trait constraints to above parsed type params
                }
            }
            None => (),
        }
        Ok(params)
    }
}

fn find_and_update_param<'sc>(
    mut params: Vec<TypeParameter<'sc>>,
    param_to_update: Pair<'sc, Rule>,
    trait_name_to_add: &'sc str,
) -> Result<(), CompileError<'sc>> {
    let mut found = false;
    for mut param in params {
        if param.name == param_to_update.as_str() {
            param.trait_constraint.push(TraitConstraint {
                name: trait_name_to_add,
            });
            found = true;
        }
    }
    if !found {
        return Err(CompileError::UndeclaredGenericTypeInWhereClause {
            span: param_to_update.as_span(),
            type_name: param_to_update.as_str(),
        });
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct TraitConstraint<'sc> {
    name: &'sc str,
}

impl<'sc> FunctionDeclaration<'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> Result<Self, CompileError<'sc>> {
        let mut parts = pair.clone().into_inner();
        let mut signature = parts.next().unwrap().into_inner();
        let _fn_keyword = signature.next().unwrap();
        let name = signature.next().unwrap().as_str();
        let mut type_params_pair = None;
        let mut where_clause_pair = None;
        let mut parameters_pair = None;
        let mut return_type_pair = None;
        while let Some(pair) = signature.next() {
            match pair.as_rule() {
                Rule::fn_decl_type_params => {
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
        let body = CodeBlock::parse_from_pair(body)?;
        Ok(FunctionDeclaration {
            name,
            parameters,
            body,
            span: pair.as_span(),
            return_type,
            type_parameters,
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionParameter<'sc> {
    name: &'sc str,
    r#type: TypeInfo<'sc>,
}

impl<'sc> FunctionParameter<'sc> {
    pub(crate) fn list_from_pairs(
        pairs: impl Iterator<Item = Pair<'sc, Rule>>,
    ) -> Result<Vec<FunctionParameter<'sc>>, CompileError<'sc>> {
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

/// Type information without an associated value, used for type inferencing and definition.
#[derive(Debug, Clone)]
pub(crate) enum TypeInfo<'sc> {
    String,
    UnsignedInteger(IntegerBits),
    Boolean,
    Generic { name: &'sc str },
    Unit,
    SelfType,
}
#[derive(Debug, Clone)]
pub(crate) enum IntegerBits {
    Eight,
    Sixteen,
    ThirtyTwo,
    SixtyFour,
}

impl<'sc> TypeInfo<'sc> {
    pub(crate) fn parse_from_pair(input: Pair<'sc, Rule>) -> Result<Self, CompileError<'sc>> {
        let mut r#type = input.into_inner();
        Self::parse_from_pair_inner(r#type.next().unwrap())
    }
    pub(crate) fn parse_from_pair_inner(input: Pair<'sc, Rule>) -> Result<Self, CompileError<'sc>> {
        Ok(match input.as_str() {
            "u8" => TypeInfo::UnsignedInteger(IntegerBits::Eight),
            "u16" => TypeInfo::UnsignedInteger(IntegerBits::Sixteen),
            "u32" => TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo),
            "u64" => TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
            "bool" => TypeInfo::Boolean,
            "string" => TypeInfo::String,
            "unit" => TypeInfo::Unit,
            other => TypeInfo::Generic { name: other },
        })
    }
}
