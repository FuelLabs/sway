use std::{
    hash::Hasher,
    slice::{Iter, IterMut},
    vec::IntoIter,
};

use crate::{engine_threading::*, type_system::priv_prelude::*};

/// A list of types that serve as the list of type params for type substitution.
/// Any types of the [TypeParam][TypeInfo::TypeParam] variant will point to an
/// index in this list.
#[derive(Debug, Clone, Default)]
pub struct SubstList {
    list: Vec<TypeParameter>,
}

impl SubstList {
    pub(crate) fn new() -> SubstList {
        SubstList { list: vec![] }
    }

    #[allow(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    #[allow(dead_code)]
    pub(crate) fn len(&self) -> usize {
        self.list.len()
    }

    #[allow(dead_code)]
    pub(crate) fn push(&mut self, type_param: TypeParameter) {
        self.list.push(type_param);
    }

    #[allow(dead_code)]
    pub(crate) fn iter(&self) -> Iter<'_, TypeParameter> {
        self.list.iter()
    }

    #[allow(dead_code)]
    pub(crate) fn into_iter(self) -> IntoIter<TypeParameter> {
        self.list.into_iter()
    }

    #[allow(dead_code)]
    pub(crate) fn iter_mut(&mut self) -> IterMut<'_, TypeParameter> {
        self.list.iter_mut()
    }
}

impl std::iter::FromIterator<TypeParameter> for SubstList {
    fn from_iter<T: IntoIterator<Item = TypeParameter>>(iter: T) -> Self {
        SubstList {
            list: iter.into_iter().collect::<Vec<TypeParameter>>(),
        }
    }
}

impl EqWithEngines for SubstList {}
impl PartialEqWithEngines for SubstList {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.list.eq(&other.list, engines)
    }
}

impl HashWithEngines for SubstList {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        self.list.hash(state, engines);
    }
}

impl OrdWithEngines for SubstList {
    fn cmp(&self, other: &Self, engines: Engines<'_>) -> std::cmp::Ordering {
        let SubstList { list: ll } = self;
        let SubstList { list: rl } = other;
        ll.cmp(rl, engines)
    }
}

impl SubstTypes for SubstList {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.list
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
    }
}
