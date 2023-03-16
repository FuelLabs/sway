use std::{collections::BTreeMap, hash::Hasher};

use itertools::Itertools;

use crate::{engine_threading::*, type_system::priv_prelude::*};

/// A collection of types that serve as the list of type params for type
/// substitution. Any types of the [TypeParam][TypeInfo::TypeParam] variant will
/// point to an element in this collection.
#[derive(Debug, Clone, Default)]
pub struct SubstList {
    list: BTreeMap<String, TypeParameter>,
}

impl SubstList {
    pub(crate) fn new() -> SubstList {
        SubstList {
            list: BTreeMap::new(),
        }
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
    pub(crate) fn insert(&mut self, name: String, type_param: TypeParameter) {
        self.list.insert(name, type_param);
    }

    pub(crate) fn elems(&self) -> Vec<&TypeParameter> {
        self.list.iter().map(|(_, type_param)| type_param).collect()
    }

    #[allow(dead_code)]
    pub(crate) fn elems_mut(&mut self) -> Vec<&mut TypeParameter> {
        self.list
            .iter_mut()
            .map(|(_, type_param)| type_param)
            .collect()
    }

    #[allow(dead_code)]
    pub(crate) fn into_elems(self) -> Vec<TypeParameter> {
        self.list.into_values().collect()
    }
}

impl EqWithEngines for SubstList {}
impl PartialEqWithEngines for SubstList {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let SubstList { list: ll } = self;
        let SubstList { list: rl } = other;
        ll.values()
            .collect_vec()
            .eq(&rl.values().collect_vec(), engines)
    }
}

impl HashWithEngines for SubstList {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let SubstList { list } = self;
        list.values().collect_vec().hash(state, engines);
    }
}

impl OrdWithEngines for SubstList {
    fn cmp(&self, other: &Self, engines: Engines<'_>) -> std::cmp::Ordering {
        let SubstList { list: ll } = self;
        let SubstList { list: rl } = other;
        ll.values()
            .collect_vec()
            .cmp(&rl.values().collect_vec(), engines)
    }
}

impl SubstTypes for SubstList {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.list
            .iter_mut()
            .for_each(|(_, x)| x.subst(type_mapping, engines));
    }
}
