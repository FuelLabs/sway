use std::sync::Arc;

use crate::{
    concurrent_slab::ConcurrentSlab,
    decl_engine::*,
    language::parsed::{
        AbiDeclaration, ConstantDeclaration, EnumDeclaration, FunctionDeclaration, ImplSelf,
        ImplTrait, StorageDeclaration, StructDeclaration, TraitDeclaration, TraitFn,
        TraitTypeDeclaration, TypeAliasDeclaration, VariableDeclaration,
    },
};

use super::parsed_id::ParsedDeclId;

/// Used inside of type inference to store declarations.
#[derive(Clone, Debug, Default)]
pub struct ParsedDeclEngine {
    variable_slab: ConcurrentSlab<VariableDeclaration>,
    function_slab: ConcurrentSlab<FunctionDeclaration>,
    trait_slab: ConcurrentSlab<TraitDeclaration>,
    trait_fn_slab: ConcurrentSlab<TraitFn>,
    trait_type_slab: ConcurrentSlab<TraitTypeDeclaration>,
    impl_trait_slab: ConcurrentSlab<ImplTrait>,
    impl_self_slab: ConcurrentSlab<ImplSelf>,
    struct_slab: ConcurrentSlab<StructDeclaration>,
    storage_slab: ConcurrentSlab<StorageDeclaration>,
    abi_slab: ConcurrentSlab<AbiDeclaration>,
    constant_slab: ConcurrentSlab<ConstantDeclaration>,
    enum_slab: ConcurrentSlab<EnumDeclaration>,
    type_alias_slab: ConcurrentSlab<TypeAliasDeclaration>,
}

pub trait ParsedDeclEngineGet<I, U> {
    fn get(&self, index: &I) -> Arc<U>;
}

pub trait ParsedDeclEngineInsert<T> {
    fn insert(&self, decl: T) -> ParsedDeclId<T>;
}

pub trait ParsedDeclEngineInsertArc<T> {
    fn insert_arc(&self, decl: Arc<T>) -> ParsedDeclId<T>;
}

pub trait ParsedDeclEngineReplace<T> {
    fn replace(&self, index: ParsedDeclId<T>, decl: T);
}

pub trait ParsedDeclEngineIndex<T>:
    ParsedDeclEngineGet<DeclId<T>, T> + ParsedDeclEngineInsert<T> + ParsedDeclEngineReplace<T>
{
}

macro_rules! decl_engine_get {
    ($slab:ident, $decl:ty) => {
        impl ParsedDeclEngineGet<ParsedDeclId<$decl>, $decl> for ParsedDeclEngine {
            fn get(&self, index: &ParsedDeclId<$decl>) -> Arc<$decl> {
                self.$slab.get(index.inner())
            }
        }
    };
}
decl_engine_get!(variable_slab, VariableDeclaration);
decl_engine_get!(function_slab, FunctionDeclaration);
decl_engine_get!(trait_slab, TraitDeclaration);
decl_engine_get!(trait_fn_slab, TraitFn);
decl_engine_get!(trait_type_slab, TraitTypeDeclaration);
decl_engine_get!(impl_trait_slab, ImplTrait);
decl_engine_get!(impl_self_slab, ImplSelf);
decl_engine_get!(struct_slab, StructDeclaration);
decl_engine_get!(storage_slab, StorageDeclaration);
decl_engine_get!(abi_slab, AbiDeclaration);
decl_engine_get!(constant_slab, ConstantDeclaration);
decl_engine_get!(enum_slab, EnumDeclaration);
decl_engine_get!(type_alias_slab, TypeAliasDeclaration);

macro_rules! decl_engine_insert {
    ($slab:ident, $decl:ty) => {
        impl ParsedDeclEngineInsert<$decl> for ParsedDeclEngine {
            fn insert(&self, decl: $decl) -> ParsedDeclId<$decl> {
                ParsedDeclId::new(self.$slab.insert(decl))
            }
        }
        impl ParsedDeclEngineInsertArc<$decl> for ParsedDeclEngine {
            fn insert_arc(&self, decl: Arc<$decl>) -> ParsedDeclId<$decl> {
                ParsedDeclId::new(self.$slab.insert_arc(decl))
            }
        }
    };
}

decl_engine_insert!(variable_slab, VariableDeclaration);
decl_engine_insert!(function_slab, FunctionDeclaration);
decl_engine_insert!(trait_slab, TraitDeclaration);
decl_engine_insert!(trait_fn_slab, TraitFn);
decl_engine_insert!(trait_type_slab, TraitTypeDeclaration);
decl_engine_insert!(impl_trait_slab, ImplTrait);
decl_engine_insert!(impl_self_slab, ImplSelf);
decl_engine_insert!(struct_slab, StructDeclaration);
decl_engine_insert!(storage_slab, StorageDeclaration);
decl_engine_insert!(abi_slab, AbiDeclaration);
decl_engine_insert!(constant_slab, ConstantDeclaration);
decl_engine_insert!(enum_slab, EnumDeclaration);
decl_engine_insert!(type_alias_slab, TypeAliasDeclaration);

macro_rules! decl_engine_clear_module {
    ($($slab:ident, $decl:ty);* $(;)?) => {
        impl ParsedDeclEngine {
            pub fn clear(&self) {
                $(
                    self.$slab.clear();
                )*
            }
        }
    };
}

decl_engine_clear_module!(
    variable_slab, VariableDeclaration;
    function_slab, FunctionDeclaration;
    trait_slab, TraitDeclaration;
    trait_fn_slab, TraitFn;
    trait_type_slab, TraitTypeDeclaration;
    impl_trait_slab, ImplTrait;
    impl_self_slab, ImplSelf;
    struct_slab, StructDeclaration;
    storage_slab, StorageDeclaration;
    abi_slab, AbiDeclaration;
    constant_slab, ConstantDeclaration;
    enum_slab, EnumDeclaration;
    type_alias_slab, TypeAliasDeclaration;
);

impl ParsedDeclEngine {
    /// Friendly helper method for calling the `get` method from the
    /// implementation of [ParsedDeclEngineGet] for [ParsedDeclEngine]
    ///
    /// Calling [ParsedDeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_function<I>(&self, index: &I) -> Arc<FunctionDeclaration>
    where
        ParsedDeclEngine: ParsedDeclEngineGet<I, FunctionDeclaration>,
    {
        self.get(index)
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [DeclEngineGet] for [DeclEngine]
    ///
    /// Calling [ParsedDeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_trait<I>(&self, index: &I) -> Arc<TraitDeclaration>
    where
        ParsedDeclEngine: ParsedDeclEngineGet<I, TraitDeclaration>,
    {
        self.get(index)
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [ParsedDeclEngineGet] for [ParsedDeclEngine]
    ///
    /// Calling [ParsedDeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_trait_fn<I>(&self, index: &I) -> Arc<TraitFn>
    where
        ParsedDeclEngine: ParsedDeclEngineGet<I, TraitFn>,
    {
        self.get(index)
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [ParsedDeclEngineGet] for [ParsedDeclEngine]
    ///
    /// Calling [ParsedDeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_impl_trait<I>(&self, index: &I) -> Arc<ImplTrait>
    where
        ParsedDeclEngine: ParsedDeclEngineGet<I, ImplTrait>,
    {
        self.get(index)
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [ParsedDeclEngineGet] for [ParsedDeclEngine]
    ///
    /// Calling [ParsedDeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_impl_self<I>(&self, index: &I) -> Arc<ImplSelf>
    where
        ParsedDeclEngine: ParsedDeclEngineGet<I, ImplSelf>,
    {
        self.get(index)
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [ParsedDeclEngineGet] for [ParsedDeclEngine]
    ///
    /// Calling [ParsedDeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_struct<I>(&self, index: &I) -> Arc<StructDeclaration>
    where
        ParsedDeclEngine: ParsedDeclEngineGet<I, StructDeclaration>,
    {
        self.get(index)
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [ParsedDeclEngineGet] for [ParsedDeclEngine].
    ///
    /// Calling [ParsedDeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_storage<I>(&self, index: &I) -> Arc<StorageDeclaration>
    where
        ParsedDeclEngine: ParsedDeclEngineGet<I, StorageDeclaration>,
    {
        self.get(index)
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [ParsedDeclEngineGet] for [ParsedDeclEngine]
    ///
    /// Calling [ParsedDeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_abi<I>(&self, index: &I) -> Arc<AbiDeclaration>
    where
        ParsedDeclEngine: ParsedDeclEngineGet<I, AbiDeclaration>,
    {
        self.get(index)
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [ParsedDeclEngineGet] for [ParsedDeclEngine]
    ///
    /// Calling [ParsedDeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_constant<I>(&self, index: &I) -> Arc<ConstantDeclaration>
    where
        ParsedDeclEngine: ParsedDeclEngineGet<I, ConstantDeclaration>,
    {
        self.get(index)
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [ParsedDeclEngineGet] for [ParsedDeclEngine]
    ///
    /// Calling [ParsedDeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_trait_type<I>(&self, index: &I) -> Arc<TraitTypeDeclaration>
    where
        ParsedDeclEngine: ParsedDeclEngineGet<I, TraitTypeDeclaration>,
    {
        self.get(index)
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [ParsedDeclEngineGet] for [ParsedDeclEngine]
    ///
    /// Calling [ParsedDeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_enum<I>(&self, index: &I) -> Arc<EnumDeclaration>
    where
        ParsedDeclEngine: ParsedDeclEngineGet<I, EnumDeclaration>,
    {
        self.get(index)
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [ParsedDeclEngineGet] for [ParsedDeclEngine]
    ///
    /// Calling [ParsedDeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_type_alias<I>(&self, index: &I) -> Arc<TypeAliasDeclaration>
    where
        ParsedDeclEngine: ParsedDeclEngineGet<I, TypeAliasDeclaration>,
    {
        self.get(index)
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [ParsedDeclEngineGet] for [ParsedDeclEngine]
    ///
    /// Calling [ParsedDeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_variable<I>(&self, index: &I) -> Arc<VariableDeclaration>
    where
        ParsedDeclEngine: ParsedDeclEngineGet<I, VariableDeclaration>,
    {
        self.get(index)
    }
}
