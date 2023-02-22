use crate::{engine_threading::*, error::*, semantic_analysis::*, type_system::*};

use sway_error::error::CompileError;

use std::{
    cmp::Ordering,
    hash::Hasher,
    slice::{Iter, IterMut},
    vec::IntoIter,
};

#[derive(Debug, Clone, Default)]
pub struct TypeParameters {
    list: Vec<TypeParameter>,
}

impl TypeParameters {
    pub fn new() -> TypeParameters {
        TypeParameters { list: vec![] }
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    fn push(&mut self, value: TypeParameter) {
        self.list.push(value);
    }

    pub(crate) fn extend(&mut self, other: TypeParameters) {
        self.list.extend(other.list);
    }

    pub fn iter(&self) -> Iter<'_, TypeParameter> {
        self.list.iter()
    }

    pub(crate) fn iter_mut(&mut self) -> IterMut<'_, TypeParameter> {
        self.list.iter_mut()
    }

    pub(crate) fn into_iter(self) -> IntoIter<TypeParameter> {
        self.list.into_iter()
    }

    /// Type check a list of [TypeParameter] and return a new list of
    /// [TypeParameter]. This will also insert this new list into the current
    /// namespace.
    pub(crate) fn type_check(
        mut ctx: TypeCheckContext,
        type_params: TypeParameters,
        disallow_trait_constraints: bool,
    ) -> CompileResult<TypeParameters> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let mut new_type_params: TypeParameters = TypeParameters::new();

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
            ok(new_type_params, warnings, errors)
        } else {
            err(warnings, errors)
        }
    }
}

impl From<Vec<TypeParameter>> for TypeParameters {
    fn from(value: Vec<TypeParameter>) -> Self {
        TypeParameters { list: value }
    }
}

impl FromIterator<TypeParameter> for TypeParameters {
    fn from_iter<I: IntoIterator<Item = TypeParameter>>(iter: I) -> Self {
        TypeParameters {
            list: iter.into_iter().collect(),
        }
    }
}

impl HashWithEngines for TypeParameters {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TypeParameters { list } = self;
        list.hash(state, engines);
    }
}

impl EqWithEngines for TypeParameters {}
impl PartialEqWithEngines for TypeParameters {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let TypeParameters { list: ll } = self;
        let TypeParameters { list: rl } = other;
        ll.eq(rl, engines)
    }
}

impl OrdWithEngines for TypeParameters {
    fn cmp(&self, other: &Self, type_engine: &TypeEngine) -> Ordering {
        let TypeParameters { list: ll } = self;
        let TypeParameters { list: rl } = other;
        ll.cmp(rl, type_engine)
            .then_with(|| ll.cmp(rl, type_engine))
    }
}

impl SubstTypes for TypeParameters {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.list
            .iter_mut()
            .for_each(|type_param| type_param.subst(type_mapping, engines));
    }
}

impl ReplaceSelfType for TypeParameters {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.list
            .iter_mut()
            .for_each(|type_param| type_param.replace_self_type(engines, self_type));
    }
}
