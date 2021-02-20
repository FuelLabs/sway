use crate::parse_tree::Expression;
use crate::{CodeBlock, CompileError, Rule};
use either::Either;
use pest::iterators::Pair;
#[derive(Debug, Clone)]
pub(crate) struct TypeParameter<'sc> {
    name: &'sc str,
    trait_constraint: Vec<TraitConstraint<'sc>>,
}

impl<'sc> TypeParameter<'sc> {
    pub(crate) fn parse_from_type_params_and_where_clause(
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
