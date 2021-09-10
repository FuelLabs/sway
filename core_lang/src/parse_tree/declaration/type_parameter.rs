use crate::build_config::BuildConfig;
use crate::span::Span;
use crate::{error::*, types::TypeInfo, Ident};
use crate::{CompileError, Rule};
use pest::iterators::Pair;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TypeParameter<'sc> {
    pub(crate) name: TypeInfo<'sc>,
    pub(crate) name_ident: Ident<'sc>,
    pub(crate) trait_constraints: Vec<TraitConstraint<'sc>>,
}

impl<'sc> TypeParameter<'sc> {
    pub(crate) fn parse_from_type_params_and_where_clause(
        type_params_pair: Option<Pair<'sc, Rule>>,
        where_clause_pair: Option<Pair<'sc, Rule>>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Vec<TypeParameter<'sc>>> {
        let path = config.map(|c| c.dir_of_code.clone());
        let mut errors = Vec::new();
        let mut warnings = vec![];
        let mut params: Vec<TypeParameter> = match type_params_pair {
            Some(type_params_pair) => {
                let mut buf = vec![];
                for pair in type_params_pair.into_inner() {
                    buf.push(TypeParameter {
                        name_ident: check!(
                            Ident::parse_from_pair(pair.clone(), config),
                            continue,
                            warnings,
                            errors
                        ),
                        name: check!(
                            TypeInfo::parse_from_pair(pair.clone(), config),
                            continue,
                            warnings,
                            errors
                        ),
                        trait_constraints: Vec::new(),
                    });
                }
                buf
            }
            None => {
                // no type params specified, ensure where clause is empty
                if let Some(where_clause_pair) = where_clause_pair {
                    return err(
                        Vec::new(),
                        vec![CompileError::UnexpectedWhereClause(Span {
                            span: where_clause_pair.as_span(),
                            path: path.clone(),
                        })],
                    );
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
                    // find associated type name
                    let param_to_edit =
                        match params.iter_mut().find(|TypeParameter { name_ident, .. }| {
                            name_ident.primary_name == type_param.as_str()
                        }) {
                            Some(o) => o,
                            None => {
                                errors.push(CompileError::ConstrainedNonExistentType {
                                    type_name: type_param.as_str().to_string(),
                                    trait_name: trait_constraint.as_str().to_string(),
                                    span: Span {
                                        span: trait_constraint.as_span(),
                                        path: path.clone(),
                                    },
                                });
                                continue;
                            }
                        };

                    param_to_edit.trait_constraints.push(TraitConstraint {
                        name: check!(
                            Ident::parse_from_pair(trait_constraint, config),
                            continue,
                            warnings,
                            errors
                        ),
                    });
                }
            }
            None => (),
        }
        ok(params, warnings, errors)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct TraitConstraint<'sc> {
    pub(crate) name: Ident<'sc>,
}
