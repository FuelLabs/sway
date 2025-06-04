use parking_lot::RwLock;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Write,
    sync::Arc,
};

use sway_types::{Named, ProgramId, SourceId, Spanned};

use crate::{
    concurrent_slab::ConcurrentSlab,
    decl_engine::{parsed_id::ParsedDeclId, *},
    engine_threading::*,
    language::{
        parsed::{
            AbiDeclaration, ConfigurableDeclaration, ConstGenericDeclaration, ConstantDeclaration,
            Declaration, EnumDeclaration, FunctionDeclaration, ImplSelfOrTrait, StorageDeclaration,
            StructDeclaration, TraitDeclaration, TraitFn, TraitTypeDeclaration,
            TypeAliasDeclaration,
        },
        ty::{
            self, TyAbiDecl, TyConfigurableDecl, TyConstGenericDecl, TyConstantDecl,
            TyDeclParsedType, TyEnumDecl, TyFunctionDecl, TyImplSelfOrTrait, TyStorageDecl,
            TyStructDecl, TyTraitDecl, TyTraitFn, TyTraitType, TyTypeAliasDecl,
        },
    },
};

/// Used inside of type inference to store declarations.
#[derive(Debug, Default)]
pub struct DeclEngine {
    function_slab: ConcurrentSlab<TyFunctionDecl>,
    trait_slab: ConcurrentSlab<TyTraitDecl>,
    trait_fn_slab: ConcurrentSlab<TyTraitFn>,
    trait_type_slab: ConcurrentSlab<TyTraitType>,
    impl_self_or_trait_slab: ConcurrentSlab<TyImplSelfOrTrait>,
    struct_slab: ConcurrentSlab<TyStructDecl>,
    storage_slab: ConcurrentSlab<TyStorageDecl>,
    abi_slab: ConcurrentSlab<TyAbiDecl>,
    constant_slab: ConcurrentSlab<TyConstantDecl>,
    configurable_slab: ConcurrentSlab<TyConfigurableDecl>,
    const_generics_slab: ConcurrentSlab<TyConstGenericDecl>,
    enum_slab: ConcurrentSlab<TyEnumDecl>,
    type_alias_slab: ConcurrentSlab<TyTypeAliasDecl>,

    function_parsed_decl_id_map:
        RwLock<HashMap<DeclId<TyFunctionDecl>, ParsedDeclId<FunctionDeclaration>>>,
    trait_parsed_decl_id_map: RwLock<HashMap<DeclId<TyTraitDecl>, ParsedDeclId<TraitDeclaration>>>,
    trait_fn_parsed_decl_id_map: RwLock<HashMap<DeclId<TyTraitFn>, ParsedDeclId<TraitFn>>>,
    trait_type_parsed_decl_id_map:
        RwLock<HashMap<DeclId<TyTraitType>, ParsedDeclId<TraitTypeDeclaration>>>,
    impl_self_or_trait_parsed_decl_id_map:
        RwLock<HashMap<DeclId<TyImplSelfOrTrait>, ParsedDeclId<ImplSelfOrTrait>>>,
    struct_parsed_decl_id_map:
        RwLock<HashMap<DeclId<TyStructDecl>, ParsedDeclId<StructDeclaration>>>,
    storage_parsed_decl_id_map:
        RwLock<HashMap<DeclId<TyStorageDecl>, ParsedDeclId<StorageDeclaration>>>,
    abi_parsed_decl_id_map: RwLock<HashMap<DeclId<TyAbiDecl>, ParsedDeclId<AbiDeclaration>>>,
    constant_parsed_decl_id_map:
        RwLock<HashMap<DeclId<TyConstantDecl>, ParsedDeclId<ConstantDeclaration>>>,
    configurable_parsed_decl_id_map:
        RwLock<HashMap<DeclId<TyConfigurableDecl>, ParsedDeclId<ConfigurableDeclaration>>>,
    const_generics_parsed_decl_id_map:
        RwLock<HashMap<DeclId<TyConstGenericDecl>, ParsedDeclId<ConstGenericDeclaration>>>,
    enum_parsed_decl_id_map: RwLock<HashMap<DeclId<TyEnumDecl>, ParsedDeclId<EnumDeclaration>>>,
    type_alias_parsed_decl_id_map:
        RwLock<HashMap<DeclId<TyTypeAliasDecl>, ParsedDeclId<TypeAliasDeclaration>>>,

    parents: RwLock<HashMap<AssociatedItemDeclId, Vec<AssociatedItemDeclId>>>,
}

impl Clone for DeclEngine {
    fn clone(&self) -> Self {
        DeclEngine {
            function_slab: self.function_slab.clone(),
            trait_slab: self.trait_slab.clone(),
            trait_fn_slab: self.trait_fn_slab.clone(),
            trait_type_slab: self.trait_type_slab.clone(),
            impl_self_or_trait_slab: self.impl_self_or_trait_slab.clone(),
            struct_slab: self.struct_slab.clone(),
            storage_slab: self.storage_slab.clone(),
            abi_slab: self.abi_slab.clone(),
            constant_slab: self.constant_slab.clone(),
            configurable_slab: self.configurable_slab.clone(),
            const_generics_slab: self.const_generics_slab.clone(),
            enum_slab: self.enum_slab.clone(),
            type_alias_slab: self.type_alias_slab.clone(),
            function_parsed_decl_id_map: RwLock::new(
                self.function_parsed_decl_id_map.read().clone(),
            ),
            trait_parsed_decl_id_map: RwLock::new(self.trait_parsed_decl_id_map.read().clone()),
            trait_fn_parsed_decl_id_map: RwLock::new(
                self.trait_fn_parsed_decl_id_map.read().clone(),
            ),
            trait_type_parsed_decl_id_map: RwLock::new(
                self.trait_type_parsed_decl_id_map.read().clone(),
            ),
            impl_self_or_trait_parsed_decl_id_map: RwLock::new(
                self.impl_self_or_trait_parsed_decl_id_map.read().clone(),
            ),
            struct_parsed_decl_id_map: RwLock::new(self.struct_parsed_decl_id_map.read().clone()),
            storage_parsed_decl_id_map: RwLock::new(self.storage_parsed_decl_id_map.read().clone()),
            abi_parsed_decl_id_map: RwLock::new(self.abi_parsed_decl_id_map.read().clone()),
            constant_parsed_decl_id_map: RwLock::new(
                self.constant_parsed_decl_id_map.read().clone(),
            ),
            configurable_parsed_decl_id_map: RwLock::new(
                self.configurable_parsed_decl_id_map.read().clone(),
            ),
            const_generics_parsed_decl_id_map: RwLock::new(
                self.const_generics_parsed_decl_id_map.read().clone(),
            ),
            enum_parsed_decl_id_map: RwLock::new(self.enum_parsed_decl_id_map.read().clone()),
            type_alias_parsed_decl_id_map: RwLock::new(
                self.type_alias_parsed_decl_id_map.read().clone(),
            ),
            parents: RwLock::new(self.parents.read().clone()),
        }
    }
}

pub trait DeclEngineGet<I, U>
{
    fn get(&self, index: &I) -> Arc<U>;
    fn map<R>(&self, index: &I, f: impl FnOnce(&U) -> R) -> R;

    // Get, clone, insert and return new item.
    // Keep the same source_id
    fn duplicate(&self, index: &I) -> DeclRef<DeclId<U>>
    where
        Self: DeclEngineInsert<U>,
        U: Named + Spanned + TyDeclParsedType + Clone
    {
        let old_item = self.get(index);
        let new_item = U::clone(&old_item);
        self.insert(new_item, None)
    }

    // Get, clone, insert and return new item.
    // Keep the same source_id
    fn map_duplicate(&self, index: &I, map: impl FnOnce(&mut U)) -> DeclRef<DeclId<U>>
    where
        Self: DeclEngineInsert<U>,
        U: Named + Spanned + TyDeclParsedType + Clone
    {
        let old_item = self.get(index);
        let mut new_item = U::clone(&old_item);
        map(&mut new_item);
        self.insert(new_item, None)
    }
}

pub trait DeclEngineGetParsedDeclId<T>
where
    T: TyDeclParsedType,
{
    fn get_parsed_decl_id(&self, decl_id: &DeclId<T>) -> Option<ParsedDeclId<T::ParsedType>>;
}

pub trait DeclEngineGetParsedDecl<T>
where
    T: TyDeclParsedType,
{
    fn get_parsed_decl(&self, decl_id: &DeclId<T>) -> Option<Declaration>;
}

pub trait DeclEngineInsert<T>
where
    T: Named + Spanned + TyDeclParsedType,
{
    fn insert(
        &self,
        decl: T,
        parsed_decl_id: Option<&ParsedDeclId<T::ParsedType>>,
    ) -> DeclRef<DeclId<T>>;
}

pub trait DeclEngineInsertArc<T>
where
    T: Named + Spanned + TyDeclParsedType,
{
    fn insert_arc(
        &self,
        decl: Arc<T>,
        parsed_decl_id: Option<&ParsedDeclId<T::ParsedType>>,
    ) -> DeclRef<DeclId<T>>;
}

pub trait DeclEngineReplace<T> {
    fn replace(&self, index: DeclId<T>, decl: T);
}

pub trait DeclEngineIndex<T>: DeclEngineGet<DeclId<T>, T> + DeclEngineReplace<T>
where
    T: Named + Spanned,
{
}

macro_rules! decl_engine_get {
    ($slab:ident, $decl:ty) => {
        impl DeclEngineGet<DeclId<$decl>, $decl> for DeclEngine {
            fn get(&self, index: &DeclId<$decl>) -> Arc<$decl> {
                self.$slab.get(index.inner())
            }

            fn map<R>(&self, index: &DeclId<$decl>, f: impl FnOnce(&$decl) -> R) -> R {
                self.$slab.map(index.inner(), f)
            }
        }

        impl DeclEngineGet<DeclRef<DeclId<$decl>>, $decl> for DeclEngine {
            fn get(&self, index: &DeclRef<DeclId<$decl>>) -> Arc<$decl> {
                self.$slab.get(index.id().inner())
            }

            fn map<R>(&self, index: &DeclRef<DeclId<$decl>>, f: impl FnOnce(&$decl) -> R) -> R {
                self.$slab.map(index.id().inner(), f)
            }
        }
    };
}
decl_engine_get!(function_slab, ty::TyFunctionDecl);
decl_engine_get!(trait_slab, ty::TyTraitDecl);
decl_engine_get!(trait_fn_slab, ty::TyTraitFn);
decl_engine_get!(trait_type_slab, ty::TyTraitType);
decl_engine_get!(impl_self_or_trait_slab, ty::TyImplSelfOrTrait);
decl_engine_get!(struct_slab, ty::TyStructDecl);
decl_engine_get!(storage_slab, ty::TyStorageDecl);
decl_engine_get!(abi_slab, ty::TyAbiDecl);
decl_engine_get!(constant_slab, ty::TyConstantDecl);
decl_engine_get!(configurable_slab, ty::TyConfigurableDecl);
decl_engine_get!(const_generics_slab, ty::TyConstGenericDecl);
decl_engine_get!(enum_slab, ty::TyEnumDecl);
decl_engine_get!(type_alias_slab, ty::TyTypeAliasDecl);

macro_rules! decl_engine_insert {
    ($slab:ident, $parsed_slab:ident, $decl:ty) => {
        impl DeclEngineInsert<$decl> for DeclEngine {
            fn insert(
                &self,
                decl: $decl,
                parsed_decl_id: Option<&ParsedDeclId<<$decl as TyDeclParsedType>::ParsedType>>,
            ) -> DeclRef<DeclId<$decl>> {
                let span = decl.span();
                let decl_name = decl.name().clone();
                let decl_id = DeclId::new(self.$slab.insert(decl));
                if let Some(parsed_decl_id) = parsed_decl_id {
                    self.$parsed_slab
                        .write()
                        .insert(decl_id, parsed_decl_id.clone());
                }
                DeclRef::new(decl_name, decl_id, span)
            }
        }
        impl DeclEngineInsertArc<$decl> for DeclEngine {
            fn insert_arc(
                &self,
                decl: Arc<$decl>,
                parsed_decl_id: Option<&ParsedDeclId<<$decl as TyDeclParsedType>::ParsedType>>,
            ) -> DeclRef<DeclId<$decl>> {
                let span = decl.span();
                let decl_name = decl.name().clone();
                let decl_id = DeclId::new(self.$slab.insert_arc(decl));
                if let Some(parsed_decl_id) = parsed_decl_id {
                    self.$parsed_slab
                        .write()
                        .insert(decl_id, parsed_decl_id.clone());
                }
                DeclRef::new(decl_name, decl_id, span)
            }
        }
    };
}
decl_engine_insert!(
    function_slab,
    function_parsed_decl_id_map,
    ty::TyFunctionDecl
);
decl_engine_insert!(trait_slab, trait_parsed_decl_id_map, ty::TyTraitDecl);
decl_engine_insert!(trait_fn_slab, trait_fn_parsed_decl_id_map, ty::TyTraitFn);
decl_engine_insert!(
    trait_type_slab,
    trait_type_parsed_decl_id_map,
    ty::TyTraitType
);
decl_engine_insert!(
    impl_self_or_trait_slab,
    impl_self_or_trait_parsed_decl_id_map,
    ty::TyImplSelfOrTrait
);
decl_engine_insert!(struct_slab, struct_parsed_decl_id_map, ty::TyStructDecl);
decl_engine_insert!(storage_slab, storage_parsed_decl_id_map, ty::TyStorageDecl);
decl_engine_insert!(abi_slab, abi_parsed_decl_id_map, ty::TyAbiDecl);
decl_engine_insert!(
    constant_slab,
    constant_parsed_decl_id_map,
    ty::TyConstantDecl
);
decl_engine_insert!(
    configurable_slab,
    configurable_parsed_decl_id_map,
    ty::TyConfigurableDecl
);
decl_engine_insert!(
    const_generics_slab,
    const_generics_parsed_decl_id_map,
    ty::TyConstGenericDecl
);
decl_engine_insert!(enum_slab, enum_parsed_decl_id_map, ty::TyEnumDecl);
decl_engine_insert!(
    type_alias_slab,
    type_alias_parsed_decl_id_map,
    ty::TyTypeAliasDecl
);

macro_rules! decl_engine_parsed_decl_id {
    ($slab:ident, $decl:ty) => {
        impl DeclEngineGetParsedDeclId<$decl> for DeclEngine {
            fn get_parsed_decl_id(
                &self,
                decl_id: &DeclId<$decl>,
            ) -> Option<ParsedDeclId<<$decl as TyDeclParsedType>::ParsedType>> {
                let parsed_decl_id_map = self.$slab.read();
                if let Some(parsed_decl_id) = parsed_decl_id_map.get(&decl_id) {
                    return Some(parsed_decl_id.clone());
                } else {
                    None
                }
            }
        }
    };
}

decl_engine_parsed_decl_id!(function_parsed_decl_id_map, ty::TyFunctionDecl);
decl_engine_parsed_decl_id!(trait_parsed_decl_id_map, ty::TyTraitDecl);
decl_engine_parsed_decl_id!(trait_fn_parsed_decl_id_map, ty::TyTraitFn);
decl_engine_parsed_decl_id!(trait_type_parsed_decl_id_map, ty::TyTraitType);
decl_engine_parsed_decl_id!(impl_self_or_trait_parsed_decl_id_map, ty::TyImplSelfOrTrait);
decl_engine_parsed_decl_id!(struct_parsed_decl_id_map, ty::TyStructDecl);
decl_engine_parsed_decl_id!(storage_parsed_decl_id_map, ty::TyStorageDecl);
decl_engine_parsed_decl_id!(abi_parsed_decl_id_map, ty::TyAbiDecl);
decl_engine_parsed_decl_id!(constant_parsed_decl_id_map, ty::TyConstantDecl);
decl_engine_parsed_decl_id!(configurable_parsed_decl_id_map, ty::TyConfigurableDecl);
decl_engine_parsed_decl_id!(enum_parsed_decl_id_map, ty::TyEnumDecl);
decl_engine_parsed_decl_id!(type_alias_parsed_decl_id_map, ty::TyTypeAliasDecl);

macro_rules! decl_engine_parsed_decl {
    ($slab:ident, $decl:ty, $ctor:expr) => {
        impl DeclEngineGetParsedDecl<$decl> for DeclEngine {
            fn get_parsed_decl(&self, decl_id: &DeclId<$decl>) -> Option<Declaration> {
                let parsed_decl_id_map = self.$slab.read();
                if let Some(parsed_decl_id) = parsed_decl_id_map.get(&decl_id) {
                    return Some($ctor(parsed_decl_id.clone()));
                } else {
                    None
                }
            }
        }
    };
}

decl_engine_parsed_decl!(
    function_parsed_decl_id_map,
    ty::TyFunctionDecl,
    Declaration::FunctionDeclaration
);
decl_engine_parsed_decl!(
    trait_parsed_decl_id_map,
    ty::TyTraitDecl,
    Declaration::TraitDeclaration
);
decl_engine_parsed_decl!(
    trait_fn_parsed_decl_id_map,
    ty::TyTraitFn,
    Declaration::TraitFnDeclaration
);
decl_engine_parsed_decl!(
    trait_type_parsed_decl_id_map,
    ty::TyTraitType,
    Declaration::TraitTypeDeclaration
);
decl_engine_parsed_decl!(
    impl_self_or_trait_parsed_decl_id_map,
    ty::TyImplSelfOrTrait,
    Declaration::ImplSelfOrTrait
);
decl_engine_parsed_decl!(
    struct_parsed_decl_id_map,
    ty::TyStructDecl,
    Declaration::StructDeclaration
);
decl_engine_parsed_decl!(
    storage_parsed_decl_id_map,
    ty::TyStorageDecl,
    Declaration::StorageDeclaration
);
decl_engine_parsed_decl!(
    abi_parsed_decl_id_map,
    ty::TyAbiDecl,
    Declaration::AbiDeclaration
);
decl_engine_parsed_decl!(
    constant_parsed_decl_id_map,
    ty::TyConstantDecl,
    Declaration::ConstantDeclaration
);
decl_engine_parsed_decl!(
    configurable_parsed_decl_id_map,
    ty::TyConfigurableDecl,
    Declaration::ConfigurableDeclaration
);
decl_engine_parsed_decl!(
    enum_parsed_decl_id_map,
    ty::TyEnumDecl,
    Declaration::EnumDeclaration
);
decl_engine_parsed_decl!(
    type_alias_parsed_decl_id_map,
    ty::TyTypeAliasDecl,
    Declaration::TypeAliasDeclaration
);

macro_rules! decl_engine_replace {
    ($slab:ident, $decl:ty) => {
        impl DeclEngineReplace<$decl> for DeclEngine {
            fn replace(&self, index: DeclId<$decl>, decl: $decl) {
                self.$slab.replace(index.inner(), decl);
            }
        }
    };
}
decl_engine_replace!(function_slab, ty::TyFunctionDecl);
decl_engine_replace!(trait_slab, ty::TyTraitDecl);
decl_engine_replace!(trait_fn_slab, ty::TyTraitFn);
decl_engine_replace!(trait_type_slab, ty::TyTraitType);
decl_engine_replace!(impl_self_or_trait_slab, ty::TyImplSelfOrTrait);
decl_engine_replace!(struct_slab, ty::TyStructDecl);
decl_engine_replace!(storage_slab, ty::TyStorageDecl);
decl_engine_replace!(abi_slab, ty::TyAbiDecl);
decl_engine_replace!(constant_slab, ty::TyConstantDecl);
decl_engine_replace!(configurable_slab, ty::TyConfigurableDecl);
decl_engine_replace!(enum_slab, ty::TyEnumDecl);
decl_engine_replace!(type_alias_slab, ty::TyTypeAliasDecl);
decl_engine_replace!(const_generics_slab, ty::TyConstGenericDecl);

macro_rules! decl_engine_index {
    ($slab:ident, $decl:ty) => {
        impl DeclEngineIndex<$decl> for DeclEngine {}
    };
}
decl_engine_index!(function_slab, ty::TyFunctionDecl);
decl_engine_index!(trait_slab, ty::TyTraitDecl);
decl_engine_index!(trait_fn_slab, ty::TyTraitFn);
decl_engine_index!(trait_type_slab, ty::TyTraitType);
decl_engine_index!(impl_self_or_trait_slab, ty::TyImplSelfOrTrait);
decl_engine_index!(struct_slab, ty::TyStructDecl);
decl_engine_index!(storage_slab, ty::TyStorageDecl);
decl_engine_index!(abi_slab, ty::TyAbiDecl);
decl_engine_index!(constant_slab, ty::TyConstantDecl);
decl_engine_index!(configurable_slab, ty::TyConfigurableDecl);
decl_engine_index!(enum_slab, ty::TyEnumDecl);
decl_engine_index!(type_alias_slab, ty::TyTypeAliasDecl);

macro_rules! decl_engine_clear_program {
    ($($slab:ident, $decl:ty);* $(;)?) => {
        impl DeclEngine {
            pub fn clear_program(&mut self, program_id: &ProgramId) {
                self.parents.write().retain(|key, _| {
                    match key {
                        AssociatedItemDeclId::TraitFn(decl_id) => {
                            self.get_trait_fn(decl_id).span().source_id().map_or(true, |src_id| &src_id.program_id() != program_id)
                        },
                        AssociatedItemDeclId::Function(decl_id) => {
                            self.get_function(decl_id).span().source_id().map_or(true, |src_id| &src_id.program_id() != program_id)
                        },
                        AssociatedItemDeclId::Type(decl_id) => {
                            self.get_type(decl_id).span().source_id().map_or(true, |src_id| &src_id.program_id() != program_id)
                        },
                        AssociatedItemDeclId::Constant(decl_id) => {
                            self.get_constant(decl_id).span().source_id().map_or(true, |src_id| &src_id.program_id() != program_id)
                        },
                    }
                });

                $(
                    self.$slab.retain(|_k, ty| match ty.span().source_id() {
                        Some(source_id) => &source_id.program_id() != program_id,
                        None => true,
                    });
                )*
            }
        }
    };
}

decl_engine_clear_program!(
    function_slab, ty::TyFunctionDecl;
    trait_slab, ty::TyTraitDecl;
    trait_fn_slab, ty::TyTraitFn;
    trait_type_slab, ty::TyTraitType;
    impl_self_or_trait_slab, ty::TyImplTrait;
    struct_slab, ty::TyStructDecl;
    storage_slab, ty::TyStorageDecl;
    abi_slab, ty::TyAbiDecl;
    constant_slab, ty::TyConstantDecl;
    configurable_slab, ty::TyConfigurableDecl;
    enum_slab, ty::TyEnumDecl;
    type_alias_slab, ty::TyTypeAliasDecl;
);

macro_rules! decl_engine_clear_module {
    ($($slab:ident, $decl:ty);* $(;)?) => {
        impl DeclEngine {
            pub fn clear_module(&mut self, source_id: &SourceId) {
                self.parents.write().retain(|key, _| {
                    match key {
                        AssociatedItemDeclId::TraitFn(decl_id) => {
                            self.get_trait_fn(decl_id).span().source_id().map_or(true, |src_id| src_id != source_id)
                        },
                        AssociatedItemDeclId::Function(decl_id) => {
                            self.get_function(decl_id).span().source_id().map_or(true, |src_id| src_id != source_id)
                        },
                        AssociatedItemDeclId::Type(decl_id) => {
                            self.get_type(decl_id).span().source_id().map_or(true, |src_id| src_id != source_id)
                        },
                        AssociatedItemDeclId::Constant(decl_id) => {
                            self.get_constant(decl_id).span().source_id().map_or(true, |src_id| src_id != source_id)
                        },
                    }
                });

                $(
                    self.$slab.retain(|_k, ty| match ty.span().source_id() {
                        Some(src_id) => src_id != source_id,
                        None => true,
                    });
                )*
            }
        }
    };
}

decl_engine_clear_module!(
    function_slab, ty::TyFunctionDecl;
    trait_slab, ty::TyTraitDecl;
    trait_fn_slab, ty::TyTraitFn;
    trait_type_slab, ty::TyTraitType;
    impl_self_or_trait_slab, ty::TyImplTrait;
    struct_slab, ty::TyStructDecl;
    storage_slab, ty::TyStorageDecl;
    abi_slab, ty::TyAbiDecl;
    constant_slab, ty::TyConstantDecl;
    configurable_slab, ty::TyConfigurableDecl;
    enum_slab, ty::TyEnumDecl;
    type_alias_slab, ty::TyTypeAliasDecl;
);

impl DeclEngine {
    /// Given a [DeclRef] `index`, finds all the parents of `index` and all the
    /// recursive parents of those parents, and so on. Does not perform
    /// duplicated computation---if the parents of a [DeclRef] have already been
    /// found, we do not find them again.
    #[allow(clippy::map_entry)]
    pub(crate) fn find_all_parents<'a, T>(
        &self,
        engines: &Engines,
        index: &'a T,
    ) -> Vec<AssociatedItemDeclId>
    where
        AssociatedItemDeclId: From<&'a T>,
    {
        let index: AssociatedItemDeclId = AssociatedItemDeclId::from(index);
        let parents = self.parents.read();
        let mut acc_parents: HashMap<AssociatedItemDeclId, AssociatedItemDeclId> = HashMap::new();
        let mut already_checked: HashSet<AssociatedItemDeclId> = HashSet::new();
        let mut left_to_check: VecDeque<AssociatedItemDeclId> = VecDeque::from([index]);
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
                            AssociatedItemDeclId::TraitFn(x_id),
                            AssociatedItemDeclId::TraitFn(curr_parent_id),
                        ) => self.get(x_id).eq(
                            &self.get(curr_parent_id),
                            &PartialEqWithEnginesContext::new(engines),
                        ),
                        (
                            AssociatedItemDeclId::Function(x_id),
                            AssociatedItemDeclId::Function(curr_parent_id),
                        ) => self.get(x_id).eq(
                            &self.get(curr_parent_id),
                            &PartialEqWithEnginesContext::new(engines),
                        ),
                        _ => false,
                    }) {
                        left_to_check.push_back(curr_parent.clone());
                    }
                }
            }
        }
        acc_parents.values().cloned().collect()
    }

    pub(crate) fn register_parent<I>(
        &self,
        index: AssociatedItemDeclId,
        parent: AssociatedItemDeclId,
    ) where
        AssociatedItemDeclId: From<DeclId<I>>,
    {
        let mut parents = self.parents.write();
        parents
            .entry(index)
            .and_modify(|e| e.push(parent.clone()))
            .or_insert_with(|| vec![parent]);
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [DeclEngineGet] for [DeclEngine]
    ///
    /// Calling [DeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_function<I>(&self, index: &I) -> Arc<ty::TyFunctionDecl>
    where
        DeclEngine: DeclEngineGet<I, ty::TyFunctionDecl>,
    {
        self.get(index)
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [DeclEngineGet] for [DeclEngine]
    ///
    /// Calling [DeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_trait<I>(&self, index: &I) -> Arc<ty::TyTraitDecl>
    where
        DeclEngine: DeclEngineGet<I, ty::TyTraitDecl>,
    {
        self.get(index)
    }

    /// Returns all the [ty::TyTraitDecl]s whose name is the same as `trait_name`.
    ///
    /// The method does a linear search over all the declared traits and is meant
    /// to be used only for diagnostic purposes.
    pub fn get_traits_by_name(&self, trait_name: &Ident) -> Vec<ty::TyTraitDecl> {
        let mut vec = vec![];
        for trait_decl in self.trait_slab.values() {
            if trait_decl.name == *trait_name {
                vec.push((*trait_decl).clone())
            }
        }
        vec
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [DeclEngineGet] for [DeclEngine]
    ///
    /// Calling [DeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_trait_fn<I>(&self, index: &I) -> Arc<ty::TyTraitFn>
    where
        DeclEngine: DeclEngineGet<I, ty::TyTraitFn>,
    {
        self.get(index)
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [DeclEngineGet] for [DeclEngine]
    ///
    /// Calling [DeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_impl_self_or_trait<I>(&self, index: &I) -> Arc<ty::TyImplSelfOrTrait>
    where
        DeclEngine: DeclEngineGet<I, ty::TyImplSelfOrTrait>,
    {
        self.get(index)
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [DeclEngineGet] for [DeclEngine]
    ///
    /// Calling [DeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_struct<I>(&self, index: &I) -> Arc<ty::TyStructDecl>
    where
        DeclEngine: DeclEngineGet<I, ty::TyStructDecl>,
    {
        self.get(index)
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [DeclEngineGet] for [DeclEngine].
    ///
    /// Calling [DeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_storage<I>(&self, index: &I) -> Arc<ty::TyStorageDecl>
    where
        DeclEngine: DeclEngineGet<I, ty::TyStorageDecl>,
    {
        self.get(index)
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [DeclEngineGet] for [DeclEngine]
    ///
    /// Calling [DeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_abi<I>(&self, index: &I) -> Arc<ty::TyAbiDecl>
    where
        DeclEngine: DeclEngineGet<I, ty::TyAbiDecl>,
    {
        self.get(index)
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [DeclEngineGet] for [DeclEngine]
    ///
    /// Calling [DeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_constant<I>(&self, index: &I) -> Arc<ty::TyConstantDecl>
    where
        DeclEngine: DeclEngineGet<I, ty::TyConstantDecl>,
    {
        self.get(index)
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [DeclEngineGet] for [DeclEngine]
    ///
    /// Calling [DeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_configurable<I>(&self, index: &I) -> Arc<ty::TyConfigurableDecl>
    where
        DeclEngine: DeclEngineGet<I, ty::TyConfigurableDecl>,
    {
        self.get(index)
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [DeclEngineGet] for [DeclEngine]
    ///
    /// Calling [DeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_const_generic<I>(&self, index: &I) -> Arc<ty::TyConstGenericDecl>
    where
        DeclEngine: DeclEngineGet<I, ty::TyConstGenericDecl>,
    {
        self.get(index)
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [DeclEngineGet] for [DeclEngine]
    ///
    /// Calling [DeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_type<I>(&self, index: &I) -> Arc<ty::TyTraitType>
    where
        DeclEngine: DeclEngineGet<I, ty::TyTraitType>,
    {
        self.get(index)
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [DeclEngineGet] for [DeclEngine]
    ///
    /// Calling [DeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_enum<I>(&self, index: &I) -> Arc<ty::TyEnumDecl>
    where
        DeclEngine: DeclEngineGet<I, ty::TyEnumDecl>,
    {
        self.get(index)
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [DeclEngineGet] for [DeclEngine]
    ///
    /// Calling [DeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_type_alias<I>(&self, index: &I) -> Arc<ty::TyTypeAliasDecl>
    where
        DeclEngine: DeclEngineGet<I, ty::TyTypeAliasDecl>,
    {
        self.get(index)
    }

    /// Pretty print method for printing the [DeclEngine]. This method is
    /// manually implemented to avoid implementation overhead regarding using
    /// [DisplayWithEngines].
    pub fn pretty_print(&self, engines: &Engines) -> String {
        let mut builder = String::new();
        let mut list = String::with_capacity(1024 * 1024);
        let funcs = self.function_slab.values();
        for (i, func) in funcs.iter().enumerate() {
            list.push_str(&format!("{i} - {:?}\n", engines.help_out(func)));
        }
        write!(builder, "DeclEngine {{\n{list}\n}}").unwrap();
        builder
    }
}
