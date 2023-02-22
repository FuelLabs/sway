use crate::{engine_threading::*, error::*, semantic_analysis::*, type_system::*};

use sway_error::error::CompileError;

use std::{
    cmp::Ordering,
    hash::Hasher,
    slice::{Iter, IterMut},
};

#[derive(Debug, Clone, Default)]
pub struct TypeParameters {
    self_type: Option<TypeParameter>,
    list: Vec<TypeParameter>,
}

impl TypeParameters {
    pub fn new() -> TypeParameters {
        TypeParameters {
            self_type: None,
            list: vec![],
        }
    }

    pub fn new_with_self_type(self_type: Option<TypeParameter>) -> TypeParameters {
        TypeParameters {
            self_type,
            list: vec![],
        }
    }

    pub(crate) fn drop_everything_but_self(self) -> TypeParameters {
        TypeParameters {
            self_type: self.self_type,
            list: vec![],
        }
    }

    pub(crate) fn to_self_type(&self) -> Option<&TypeParameter> {
        self.self_type.as_ref()
    }

    pub fn to_mut_self_type(&mut self) -> Option<&mut TypeParameter> {
        self.self_type.as_mut()
    }

    pub fn to_list_excluding_self(&self) -> &[TypeParameter] {
        &self.list
    }

    pub fn is_empty_excluding_self(&self) -> bool {
        self.list.is_empty()
    }

    pub fn len_excluding_self(&self) -> usize {
        self.list.len()
    }

    pub(crate) fn extend_excluding_self(&mut self, other: TypeParameters) {
        self.list.extend(other.list);
    }

    pub fn iter_excluding_self(&self) -> Iter<'_, TypeParameter> {
        self.list.iter()
    }

    pub(crate) fn iter_mut_excluding_self(&mut self) -> IterMut<'_, TypeParameter> {
        self.list.iter_mut()
    }

    pub fn iter_including_self(&self) -> TypeParametersIter<'_> {
        TypeParametersIter::new(&self.self_type, false, self.list.iter())
    }

    /// Type check a [TypeParameters] and return a new [TypeParameters]. This
    /// will also insert this new list into the current namespace.
    pub(crate) fn type_check(
        mut ctx: TypeCheckContext,
        type_params: Vec<TypeParameter>,
        disallow_trait_constraints: bool,
        self_type_param: Option<TypeParameter>,
    ) -> CompileResult<TypeParameters> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let mut new_type_params: Vec<TypeParameter> = vec![];

        if let Some(self_type_param) = self_type_param.clone() {
            self_type_param.insert_self_type_into_namespace(ctx.by_ref());
        }

        for type_param in type_params.into_iter() {
            if disallow_trait_constraints && !type_param.trait_constraints.is_empty() {
                let errors = vec![CompileError::WhereClauseNotYetSupported {
                    span: type_param.trait_constraints_span,
                }];
                return err(vec![], errors);
            }
            new_type_params.push(check!(
                TypeParameter::type_check(ctx.by_ref(), type_param),
                continue,
                warnings,
                errors
            ));
        }

        if errors.is_empty() {
            ok(
                TypeParameters {
                    self_type: self_type_param,
                    list: new_type_params,
                },
                warnings,
                errors,
            )
        } else {
            err(warnings, errors)
        }
    }
}

impl From<Vec<TypeParameter>> for TypeParameters {
    fn from(value: Vec<TypeParameter>) -> Self {
        TypeParameters {
            self_type: None,
            list: value,
        }
    }
}

impl FromIterator<TypeParameter> for TypeParameters {
    fn from_iter<I: IntoIterator<Item = TypeParameter>>(iter: I) -> Self {
        TypeParameters {
            self_type: None,
            list: iter.into_iter().collect(),
        }
    }
}

impl HashWithEngines for TypeParameters {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TypeParameters { self_type, list } = self;
        self_type.hash(state, engines);
        list.hash(state, engines);
    }
}

impl EqWithEngines for TypeParameters {}
impl PartialEqWithEngines for TypeParameters {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let TypeParameters {
            self_type: lst,
            list: ll,
        } = self;
        let TypeParameters {
            self_type: rst,
            list: rl,
        } = other;
        lst.eq(rst, engines) && ll.eq(rl, engines)
    }
}

impl OrdWithEngines for TypeParameters {
    fn cmp(&self, other: &Self, type_engine: &TypeEngine) -> Ordering {
        let TypeParameters {
            self_type: lst,
            list: ll,
        } = self;
        let TypeParameters {
            self_type: rst,
            list: rl,
        } = other;
        lst.cmp(rst, type_engine)
            .then_with(|| ll.cmp(rl, type_engine))
    }
}

impl SubstTypes for TypeParameters {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        if let Some(type_param) = self.self_type.as_mut() {
            type_param.subst(type_mapping, engines);
        }
        self.list
            .iter_mut()
            .for_each(|type_param| type_param.subst(type_mapping, engines));
    }
}
