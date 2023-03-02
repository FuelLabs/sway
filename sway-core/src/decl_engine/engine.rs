use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::RwLock,
};

use sway_types::{Named, Spanned};

use crate::{
    concurrent_slab::ConcurrentSlab,
    decl_engine::*,
    engine_threading::*,
    language::ty::{
        self, TyAbiDeclaration, TyConstantDeclaration, TyEnumDeclaration, TyFunctionDeclaration,
        TyImplTrait, TyStorageDeclaration, TyStructDeclaration, TyTraitDeclaration, TyTraitFn,
    },
};

/// Used inside of type inference to store declarations.
#[derive(Debug, Default)]
pub struct DeclEngine {
    function_slab: ConcurrentSlab<TyFunctionDeclaration>,
    trait_slab: ConcurrentSlab<TyTraitDeclaration>,
    trait_fn_slab: ConcurrentSlab<TyTraitFn>,
    impl_trait_slab: ConcurrentSlab<TyImplTrait>,
    struct_slab: ConcurrentSlab<TyStructDeclaration>,
    storage_slab: ConcurrentSlab<TyStorageDeclaration>,
    abi_slab: ConcurrentSlab<TyAbiDeclaration>,
    constant_slab: ConcurrentSlab<TyConstantDeclaration>,
    enum_slab: ConcurrentSlab<TyEnumDeclaration>,

    parents: RwLock<HashMap<FunctionalDeclId, Vec<FunctionalDeclId>>>,
}

pub trait DeclEngineIndex<T>
where
    T: Named + Spanned,
{
    fn get(&self, index: DeclId<T>) -> T;
    fn replace(&self, index: DeclId<T>, decl: T);
    fn insert(&self, decl: T) -> DeclRef<DeclId<T>>;
}

macro_rules! decl_engine_index {
    ($slab:ident, $decl:ty) => {
        impl DeclEngineIndex<$decl> for DeclEngine {
            fn get(&self, index: DeclId<$decl>) -> $decl {
                self.$slab.get(index.inner())
            }

            fn replace(&self, index: DeclId<$decl>, decl: $decl) {
                self.$slab.replace(index, decl);
            }

            fn insert(&self, decl: $decl) -> DeclRef<DeclId<$decl>> {
                let span = decl.span();
                DeclRef {
                    name: decl.name().clone(),
                    id: DeclId::new(self.$slab.insert(decl)),
                    decl_span: span,
                }
            }
        }
    };
}
decl_engine_index!(function_slab, ty::TyFunctionDeclaration);
decl_engine_index!(trait_slab, ty::TyTraitDeclaration);
decl_engine_index!(trait_fn_slab, ty::TyTraitFn);
decl_engine_index!(impl_trait_slab, ty::TyImplTrait);
decl_engine_index!(struct_slab, ty::TyStructDeclaration);
decl_engine_index!(storage_slab, ty::TyStorageDeclaration);
decl_engine_index!(abi_slab, ty::TyAbiDeclaration);
decl_engine_index!(constant_slab, ty::TyConstantDeclaration);
decl_engine_index!(enum_slab, ty::TyEnumDeclaration);

impl DeclEngine {
    /// Given a [DeclRef] `index`, finds all the parents of `index` and all the
    /// recursive parents of those parents, and so on. Does not perform
    /// duplicated computation---if the parents of a [DeclRef] have already been
    /// found, we do not find them again.
    #[allow(clippy::map_entry)]
    pub(crate) fn find_all_parents<'a, T>(
        &self,
        engines: Engines<'_>,
        index: &'a T,
    ) -> Vec<FunctionalDeclId>
    where
        FunctionalDeclId: From<&'a T>,
    {
        let index: FunctionalDeclId = FunctionalDeclId::from(index);
        let parents = self.parents.read().unwrap();
        let mut acc_parents: HashMap<FunctionalDeclId, FunctionalDeclId> = HashMap::new();
        let mut already_checked: HashSet<FunctionalDeclId> = HashSet::new();
        let mut left_to_check: VecDeque<FunctionalDeclId> = VecDeque::from([index]);
        while let Some(curr) = left_to_check.pop_front() {
            if !already_checked.insert(curr.clone()) {
                continue;
            }
            if let Some(curr_parents) = parents.get(&curr) {
                for curr_parent in curr_parents.iter() {
                    if !acc_parents.contains_key(curr_parent) {
                        acc_parents.insert(curr_parent.clone(), curr_parent.clone());
                    }
                    if !left_to_check.iter().any(|x| match (x, curr_parent) {
                        (
                            FunctionalDeclId::TraitFn(x_id),
                            FunctionalDeclId::TraitFn(curr_parent_id),
                        ) => self.get(*x_id).eq(&self.get(*curr_parent_id), engines),
                        (
                            FunctionalDeclId::Function(x_id),
                            FunctionalDeclId::Function(curr_parent_id),
                        ) => self.get(*x_id).eq(&self.get(*curr_parent_id), engines),
                        _ => false,
                    }) {
                        left_to_check.push_back(curr_parent.clone());
                    }
                }
            }
        }
        acc_parents.values().cloned().collect()
    }

    pub(crate) fn register_parent<I>(&self, index: FunctionalDeclId, parent: FunctionalDeclId)
    where
        FunctionalDeclId: From<DeclId<I>>,
    {
        let mut parents = self.parents.write().unwrap();
        parents
            .entry(index)
            .and_modify(|e| e.push(parent.clone()))
            .or_insert_with(|| vec![parent]);
    }

    pub fn get_function<'a, T, I>(&self, index: &'a T) -> ty::TyFunctionDeclaration
    where
        DeclId<I>: From<&'a T>,
    {
        self.function_slab.get(DeclId::from(index).inner())
    }

    pub fn get_trait<'a, T, I>(&self, index: &'a T) -> ty::TyTraitDeclaration
    where
        DeclId<I>: From<&'a T>,
    {
        self.trait_slab.get(DeclId::from(index).inner())
    }

    pub fn get_trait_fn<'a, T, I>(&self, index: &'a T) -> ty::TyTraitFn
    where
        DeclId<I>: From<&'a T>,
    {
        self.trait_fn_slab.get(DeclId::from(index).inner())
    }

    pub fn get_impl_trait<'a, T, I>(&self, index: &'a T) -> ty::TyImplTrait
    where
        DeclId<I>: From<&'a T>,
    {
        self.impl_trait_slab.get(DeclId::from(index).inner())
    }

    pub fn get_struct<'a, T, I>(&self, index: &'a T) -> ty::TyStructDeclaration
    where
        DeclId<I>: From<&'a T>,
    {
        self.struct_slab.get(DeclId::from(index).inner())
    }

    pub fn get_storage<'a, T, I>(&self, index: &'a T) -> ty::TyStorageDeclaration
    where
        DeclId<I>: From<&'a T>,
    {
        self.storage_slab.get(DeclId::from(index).inner())
    }

    pub fn get_abi<'a, T, I>(&self, index: &'a T) -> ty::TyAbiDeclaration
    where
        DeclId<I>: From<&'a T>,
    {
        self.abi_slab.get(DeclId::from(index).inner())
    }

    pub fn get_constant<'a, T, I>(&self, index: &'a T) -> ty::TyConstantDeclaration
    where
        DeclId<I>: From<&'a T>,
    {
        self.constant_slab.get(DeclId::from(index).inner())
    }

    pub fn get_enum<'a, T, I>(&self, index: &'a T) -> ty::TyEnumDeclaration
    where
        DeclId<I>: From<&'a T>,
    {
        self.enum_slab.get(DeclId::from(index).inner())
    }
}
