use std::hash::Hasher;

use crate::{engine_threading::*, type_system::priv_prelude::*};

/// A collection of types that serve as the list of type params for type
/// substitution. Any types of the [TypeParam][TypeInfo::TypeParam] variant will
/// point to an element in this collection.
#[derive(Debug, Clone, Default)]
pub struct SubstList {
    list: Vec<TypeParameter>,
}

impl SubstList {
    pub(crate) fn new() -> SubstList {
        SubstList { list: vec![] }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub(crate) fn len(&self) -> usize {
        self.list.len()
    }

    pub(crate) fn push(&mut self, type_param: TypeParameter) {
        self.list.push(type_param);
    }

    pub(crate) fn elems(&self) -> Vec<&TypeParameter> {
        self.list.iter().collect()
    }

    pub(crate) fn elems_mut(&mut self) -> Vec<&mut TypeParameter> {
        self.list.iter_mut().collect()
    }

    pub(crate) fn into_elems(self) -> Vec<TypeParameter> {
        self.list.into_iter().collect()
    }

    pub(crate) fn index(&self, index: usize) -> Option<&TypeParameter> {
        self.list.get(index)
    }

    pub(crate) fn apply_type_args(&mut self, type_args: &[TypeArgument]) {
        self.list
            .iter_mut()
            .zip(type_args.iter())
            .for_each(|(type_param, type_arg)| {
                type_param.type_id = type_arg.type_id;
            });
    }
}

impl EqWithEngines for SubstList {}
impl PartialEqWithEngines for SubstList {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let SubstList { list: ll } = self;
        let SubstList { list: rl } = other;
        ll.eq(rl, engines)
    }
}

impl HashWithEngines for SubstList {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let SubstList { list } = self;
        list.hash(state, engines);
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

impl CreateCopy<SubstList> for SubstList {
    fn scoped_copy(&self, engines: Engines<'_>) -> Self {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        let list = self
            .list
            .clone()
            .into_iter()
            .map(|mut type_param| {
                type_param.type_id =
                    type_engine.insert(decl_engine, TypeInfo::Placeholder(type_param.clone()));
                type_param
            })
            .collect();
        SubstList { list }
    }

    fn unscoped_copy(&self) -> Self {
        self.clone()
    }
}
