use crate::{
    build_config::BuildConfig, error::*, parse_tree::ident, type_engine::*, CompileError, Rule,
    TypedDeclaration,
};

use sway_types::{ident::Ident, span::Span};

use pest::iterators::Pair;
use std::convert::From;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeParameter {
    pub(crate) type_id: TypeId,
    pub(crate) name_ident: Ident,
    pub(crate) trait_constraints: Vec<TraitConstraint>,
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
                errors.push(CompileError::UnexpectedWhereClause(Span {
                    span: where_clause_pair.as_span(),
                    path,
                }));
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
                let mut pair = where_clause_pair.into_inner().peekable();
                while pair.peek().is_some() {
                    let type_param = ident::parse_from_pair(pair.next().unwrap(), config)
                        .value
                        .unwrap();
                    let trait_constraint = ident::parse_from_pair(pair.next().unwrap(), config)
                        .value
                        .unwrap();
                    // assign trait constraints to above parsed type params
                    // find associated type name
                    let param_to_edit =
                        match params.iter_mut().find(|TypeParameter { name_ident, .. }| {
                            name_ident.as_str() == type_param.as_str()
                        }) {
                            Some(o) => o,
                            None => {
                                errors.push(CompileError::ConstrainedNonExistentType {
                                    type_name: type_param,
                                    trait_name: trait_constraint.clone(),
                                    span: trait_constraint.span().clone(),
                                });
                                continue;
                            }
                        };

                    param_to_edit.trait_constraints.push(TraitConstraint {
                        name: trait_constraint,
                    });
                }
                params
            }
        };
        ok(params, warnings, errors)
    }

    pub(crate) fn parse_from_type_params(
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
