use crate::{
    build_config::BuildConfig, error::*, parse_tree::ident, type_engine::*, CompileError, Rule,
    TypedDeclaration,
};

use sway_types::{ident::Ident, span::Span};

use pest::iterators::Pair;
use std::{
    convert::From,
    hash::{Hash, Hasher},
};

#[derive(Debug, Clone, Eq)]
pub struct TypeParameter {
    pub(crate) type_id: TypeId,
    pub(crate) name_ident: Ident,
    pub(crate) trait_constraints: Vec<TraitConstraint>,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl Hash for TypeParameter {
    fn hash<H: Hasher>(&self, state: &mut H) {
        look_up_type_id(self.type_id).hash(state);
        self.name_ident.hash(state);
        self.trait_constraints.hash(state);
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypeParameter {
    fn eq(&self, other: &Self) -> bool {
        look_up_type_id(self.type_id) == look_up_type_id(other.type_id)
            && self.name_ident == other.name_ident
            && self.trait_constraints == other.trait_constraints
    }
}

impl From<&TypeParameter> for TypedDeclaration {
    fn from(n: &TypeParameter) -> Self {
        TypedDeclaration::GenericTypeForFunctionScope {
            name: n.name_ident.clone(),
        }
    }
}

impl TypeParameter {
    pub(crate) fn parse_from_type_params_and_where_clause(
        type_params_pair: Option<Pair<Rule>>,
        where_clause_pair: Option<Pair<Rule>>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Vec<TypeParameter>> {
        let path = config.map(|c| c.path());
        let mut warnings = vec![];
        let mut errors = vec![];
        let params = match (type_params_pair, where_clause_pair) {
            (None, None) => vec![],
            (None, Some(where_clause_pair)) => {
                errors.push(CompileError::UnexpectedWhereClause(Span::from_pest(
                    where_clause_pair.as_span(),
                    path,
                )));
                return err(warnings, errors);
            }
            (Some(type_params_pair), None) => check!(
                Self::parse_from_type_params(type_params_pair, config),
                vec!(),
                warnings,
                errors
            ),
            (Some(type_params_pair), Some(where_clause_pair)) => {
                let mut params = check!(
                    Self::parse_from_type_params(type_params_pair, config),
                    vec!(),
                    warnings,
                    errors
                );
                let where_clauses = check!(
                    WhereClause::parse_from_trait_bounds(where_clause_pair, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                for where_clause in where_clauses.into_iter() {
                    let param_to_edit =
                        match params.iter_mut().find(|TypeParameter { name_ident, .. }| {
                            name_ident.as_str() == where_clause.type_param.as_str()
                        }) {
                            Some(o) => o,
                            None => {
                                errors.push(CompileError::ConstrainedNonExistentType {
                                    type_name: where_clause.type_param,
                                    trait_name: where_clause.trait_constraint.clone(),
                                    span: where_clause.trait_constraint.span().clone(),
                                });
                                continue;
                            }
                        };

                    param_to_edit.trait_constraints.push(TraitConstraint {
                        name: where_clause.trait_constraint,
                    });
                }
                params
            }
        };
        ok(params, warnings, errors)
    }

    fn parse_from_type_params(
        type_params_pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Vec<Self>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut buf = vec![];
        for pair in type_params_pair.into_inner() {
            let name_ident = check!(
                ident::parse_from_pair(pair.clone(), config),
                continue,
                warnings,
                errors
            );
            let type_id = insert_type(check!(
                TypeInfo::parse_from_type_param_pair(pair.clone(), config),
                continue,
                warnings,
                errors
            ));
            buf.push(TypeParameter {
                name_ident,
                type_id,
                trait_constraints: Vec::new(),
            });
        }
        ok(buf, warnings, errors)
    }

    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.type_id = match look_up_type_id(self.type_id).matches_type_parameter(type_mapping) {
            Some(matching_id) => insert_type(TypeInfo::Ref(matching_id)),
            None => insert_type(look_up_type_id_raw(self.type_id)),
        };
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct TraitConstraint {
    pub(crate) name: Ident,
}

pub(crate) struct WhereClause {
    pub(crate) type_param: Ident,
    pub(crate) trait_constraint: Ident,
}

impl WhereClause {
    pub(crate) fn parse_from_trait_bounds(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Vec<WhereClause>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut iter = pair.into_inner().peekable();
        let mut clauses = vec![];
        while iter.peek().is_some() {
            let type_param = check!(
                ident::parse_from_pair(iter.next().unwrap(), config),
                continue,
                warnings,
                errors
            );
            let trait_constraint = check!(
                ident::parse_from_pair(iter.next().unwrap(), config),
                continue,
                warnings,
                errors
            );
            clauses.push(WhereClause {
                type_param,
                trait_constraint,
            });
        }
        ok(clauses, warnings, errors)
    }
}
