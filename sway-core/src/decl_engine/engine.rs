use sway_types::{Named, Spanned};

use crate::{
    concurrent_slab::ConcurrentSlab,
    decl_engine::*,
    engine_threading::*,
    language::ty::{
        self, TyAbiDeclaration, TyConstantDeclaration, TyEnumDeclaration, TyFunctionDeclaration,
        TyImplTrait, TyStorageDeclaration, TyStructDeclaration, TyTraitDeclaration, TyTraitFn,
    },
    type_system::TypeSubstList,
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
}

pub trait DeclEngineIndex<T>
where
    T: Named + Spanned,
{
    fn get(&self, index: DeclId<T>) -> T;
    fn replace(&self, index: DeclId<T>, decl: T);
    fn insert(&self, decl: T, subst_list: TypeSubstList) -> DeclRef<DeclId<T>>;
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

            fn insert(&self, decl: $decl, subst_list: TypeSubstList) -> DeclRef<DeclId<$decl>> {
                let span = decl.span();
                DeclRef::new(
                    decl.name().clone(),
                    DeclId::new(self.$slab.insert(decl)),
                    subst_list,
                    span,
                )
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
        _engines: Engines<'_>,
        _index: &'a T,
    ) -> Vec<FunctionalDeclId>
    where
        FunctionalDeclId: From<&'a T>,
    {
        todo!()
    }

    pub fn get_function<'a, T>(&self, index: &'a T) -> ty::TyFunctionDeclaration
    where
        DeclId<ty::TyFunctionDeclaration>: From<&'a T>,
    {
        self.function_slab.get(DeclId::from(index).inner())
    }

    pub fn get_trait<'a, T>(&self, index: &'a T) -> ty::TyTraitDeclaration
    where
        DeclId<ty::TyTraitDeclaration>: From<&'a T>,
    {
        self.trait_slab.get(DeclId::from(index).inner())
    }

    pub fn get_trait_fn<'a, T>(&self, index: &'a T) -> ty::TyTraitFn
    where
        DeclId<ty::TyTraitFn>: From<&'a T>,
    {
        self.trait_fn_slab.get(DeclId::from(index).inner())
    }

    pub fn get_impl_trait<'a, T>(&self, index: &'a T) -> ty::TyImplTrait
    where
        DeclId<ty::TyImplTrait>: From<&'a T>,
    {
        self.impl_trait_slab.get(DeclId::from(index).inner())
    }

    pub fn get_struct<'a, T>(&self, index: &'a T) -> ty::TyStructDeclaration
    where
        DeclId<ty::TyStructDeclaration>: From<&'a T>,
    {
        self.struct_slab.get(DeclId::from(index).inner())
    }

    pub fn get_storage<'a, T>(&self, index: &'a T) -> ty::TyStorageDeclaration
    where
        DeclId<ty::TyStorageDeclaration>: From<&'a T>,
    {
        self.storage_slab.get(DeclId::from(index).inner())
    }

    pub fn get_abi<'a, T>(&self, index: &'a T) -> ty::TyAbiDeclaration
    where
        DeclId<ty::TyAbiDeclaration>: From<&'a T>,
    {
        self.abi_slab.get(DeclId::from(index).inner())
    }

    pub fn get_constant<'a, T>(&self, index: &'a T) -> ty::TyConstantDeclaration
    where
        DeclId<ty::TyConstantDeclaration>: From<&'a T>,
    {
        self.constant_slab.get(DeclId::from(index).inner())
    }

    pub fn get_enum<'a, T>(&self, index: &'a T) -> ty::TyEnumDeclaration
    where
        DeclId<ty::TyEnumDeclaration>: From<&'a T>,
    {
        self.enum_slab.get(DeclId::from(index).inner())
    }
}
