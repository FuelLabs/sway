use crate::{
    build_config::BuildConfig, error::*, parse_tree::ident, type_engine::*, CompileError, Rule,
    TypedDeclaration,
};

use sway_types::{ident::Ident, span::Span};

use pest::iterators::Pair;
use std::convert::From;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TypeParameter {
    pub(crate) name: TypeInfo,
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
        let mut errors = Vec::new();
        let mut warnings = vec![];
        let mut params: Vec<TypeParameter> = match type_params_pair {
            Some(type_params_pair) => {
                let mut buf = vec![];
                for pair in type_params_pair.into_inner() {
                    buf.push(TypeParameter {
                        name_ident: check!(
                            ident::parse_from_pair(pair.clone(), config),
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
                            path,
                        })],
                    );
                }
                Vec::new()
            }
        };
        if let Some(where_clause_pair) = where_clause_pair {
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
        }
        ok(params, warnings, errors)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct TraitConstraint {
    pub(crate) name: Ident,
}
