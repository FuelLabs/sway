use crate::{
    concurrent_slab::ConcurrentSlab,
    language::parsed::{
        AbiDeclaration, ConfigurableDeclaration, ConstGenericDeclaration, ConstantDeclaration,
        EnumDeclaration, EnumVariant, FunctionDeclaration, ImplSelfOrTrait, StorageDeclaration,
        StructDeclaration, TraitDeclaration, TraitFn, TraitTypeDeclaration, TypeAliasDeclaration,
        VariableDeclaration,
    },
};

use std::sync::Arc;
use sway_types::{ProgramId, SourceId, Spanned};

use super::parsed_id::ParsedDeclId;

/// Used inside of type inference to store declarations.
#[derive(Clone, Debug, Default)]
pub struct ParsedDeclEngine {
    variable_slab: ConcurrentSlab<VariableDeclaration>,
    function_slab: ConcurrentSlab<FunctionDeclaration>,
    trait_slab: ConcurrentSlab<TraitDeclaration>,
    trait_fn_slab: ConcurrentSlab<TraitFn>,
    trait_type_slab: ConcurrentSlab<TraitTypeDeclaration>,
    impl_self_or_trait_slab: ConcurrentSlab<ImplSelfOrTrait>,
    struct_slab: ConcurrentSlab<StructDeclaration>,
    storage_slab: ConcurrentSlab<StorageDeclaration>,
    abi_slab: ConcurrentSlab<AbiDeclaration>,
    constant_slab: ConcurrentSlab<ConstantDeclaration>,
    configurable_slab: ConcurrentSlab<ConfigurableDeclaration>,
    const_generic_slab: ConcurrentSlab<ConstGenericDeclaration>,
    enum_slab: ConcurrentSlab<EnumDeclaration>,
    enum_variant_slab: ConcurrentSlab<EnumVariant>,
    type_alias_slab: ConcurrentSlab<TypeAliasDeclaration>,
}

pub trait ParsedDeclEngineGet<I, U> {
    fn get(&self, index: &I) -> Arc<U>;
    fn map<R>(&self, index: &I, f: impl FnOnce(&U) -> R) -> R;
}

pub trait ParsedDeclEngineInsert<T> {
    fn insert(&self, decl: T) -> ParsedDeclId<T>;
}

#[allow(unused)]
pub trait ParsedDeclEngineReplace<T> {
    fn replace(&self, index: ParsedDeclId<T>, decl: T);
}

#[allow(unused)]
pub trait ParsedDeclEngineIndex<T>:
    ParsedDeclEngineGet<ParsedDeclId<T>, T> + ParsedDeclEngineInsert<T> + ParsedDeclEngineReplace<T>
{
}

macro_rules! decl_engine_get {
    ($slab:ident, $decl:ty) => {
        impl ParsedDeclEngineGet<ParsedDeclId<$decl>, $decl> for ParsedDeclEngine {
            fn get(&self, index: &ParsedDeclId<$decl>) -> Arc<$decl> {
                self.$slab.get(index.inner())
            }

            fn map<R>(&self, index: &ParsedDeclId<$decl>, f: impl FnOnce(&$decl) -> R) -> R {
                self.$slab.map(index.inner(), f)
            }
        }
    };
}
decl_engine_get!(variable_slab, VariableDeclaration);
decl_engine_get!(function_slab, FunctionDeclaration);
decl_engine_get!(trait_slab, TraitDeclaration);
decl_engine_get!(trait_fn_slab, TraitFn);
decl_engine_get!(trait_type_slab, TraitTypeDeclaration);
decl_engine_get!(impl_self_or_trait_slab, ImplSelfOrTrait);
decl_engine_get!(struct_slab, StructDeclaration);
decl_engine_get!(storage_slab, StorageDeclaration);
decl_engine_get!(abi_slab, AbiDeclaration);
decl_engine_get!(constant_slab, ConstantDeclaration);
decl_engine_get!(configurable_slab, ConfigurableDeclaration);
decl_engine_get!(const_generic_slab, ConstGenericDeclaration);
decl_engine_get!(enum_slab, EnumDeclaration);
decl_engine_get!(enum_variant_slab, EnumVariant);
decl_engine_get!(type_alias_slab, TypeAliasDeclaration);

macro_rules! decl_engine_insert {
    ($slab:ident, $decl:ty) => {
        impl ParsedDeclEngineInsert<$decl> for ParsedDeclEngine {
            fn insert(&self, decl: $decl) -> ParsedDeclId<$decl> {
                ParsedDeclId::new(self.$slab.insert(decl))
            }
        }
    };
}

decl_engine_insert!(variable_slab, VariableDeclaration);
decl_engine_insert!(function_slab, FunctionDeclaration);
decl_engine_insert!(trait_slab, TraitDeclaration);
decl_engine_insert!(trait_fn_slab, TraitFn);
decl_engine_insert!(trait_type_slab, TraitTypeDeclaration);
decl_engine_insert!(impl_self_or_trait_slab, ImplSelfOrTrait);
decl_engine_insert!(struct_slab, StructDeclaration);
decl_engine_insert!(storage_slab, StorageDeclaration);
decl_engine_insert!(abi_slab, AbiDeclaration);
decl_engine_insert!(constant_slab, ConstantDeclaration);
decl_engine_insert!(configurable_slab, ConfigurableDeclaration);
decl_engine_insert!(const_generic_slab, ConstGenericDeclaration);
decl_engine_insert!(enum_slab, EnumDeclaration);
decl_engine_insert!(enum_variant_slab, EnumVariant);
decl_engine_insert!(type_alias_slab, TypeAliasDeclaration);

macro_rules! decl_engine_replace {
    ($slab:ident, $decl:ty) => {
        impl ParsedDeclEngineReplace<$decl> for ParsedDeclEngine {
            fn replace(&self, index: ParsedDeclId<$decl>, decl: $decl) {
                self.$slab.replace(index.inner(), decl);
            }
        }
    };
}

decl_engine_replace!(variable_slab, VariableDeclaration);
decl_engine_replace!(function_slab, FunctionDeclaration);
decl_engine_replace!(trait_slab, TraitDeclaration);
decl_engine_replace!(trait_fn_slab, TraitFn);
decl_engine_replace!(trait_type_slab, TraitTypeDeclaration);
decl_engine_replace!(impl_self_or_trait_slab, ImplSelfOrTrait);
decl_engine_replace!(struct_slab, StructDeclaration);
decl_engine_replace!(storage_slab, StorageDeclaration);
decl_engine_replace!(abi_slab, AbiDeclaration);
decl_engine_replace!(configurable_slab, ConfigurableDeclaration);
decl_engine_replace!(constant_slab, ConstantDeclaration);
decl_engine_replace!(enum_slab, EnumDeclaration);
decl_engine_replace!(type_alias_slab, TypeAliasDeclaration);

macro_rules! decl_engine_clear {
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

macro_rules! decl_engine_index {
    ($slab:ident, $decl:ty) => {
        impl ParsedDeclEngineIndex<$decl> for ParsedDeclEngine {}
    };
}
decl_engine_index!(variable_slab, VariableDeclaration);
decl_engine_index!(function_slab, FunctionDeclaration);
decl_engine_index!(trait_slab, TraitDeclaration);
decl_engine_index!(trait_fn_slab, TraitFn);
decl_engine_index!(trait_type_slab, TraitTypeDeclaration);
decl_engine_index!(impl_self_or_trait_slab, ImplSelfOrTrait);
decl_engine_index!(struct_slab, StructDeclaration);
decl_engine_index!(storage_slab, StorageDeclaration);
decl_engine_index!(abi_slab, AbiDeclaration);
decl_engine_index!(configurable_slab, ConfigurableDeclaration);
decl_engine_index!(constant_slab, ConstantDeclaration);
decl_engine_index!(enum_slab, EnumDeclaration);
decl_engine_index!(type_alias_slab, TypeAliasDeclaration);

decl_engine_clear!(
    variable_slab, VariableDeclaration;
    function_slab, FunctionDeclaration;
    trait_slab, TraitDeclaration;
    trait_fn_slab, TraitFn;
    trait_type_slab, TraitTypeDeclaration;
    impl_self_or_trait_slab, ImplTrait;
    struct_slab, StructDeclaration;
    storage_slab, StorageDeclaration;
    abi_slab, AbiDeclaration;
    constant_slab, ConstantDeclaration;
    enum_slab, EnumDeclaration;
    type_alias_slab, TypeAliasDeclaration;
);

macro_rules! decl_engine_clear_program {
    ($(($slab:ident, $getter:expr)),* $(,)?) => {
        impl ParsedDeclEngine {
            pub fn clear_program(&mut self, program_id: &ProgramId) {
                $(
                    self.$slab.retain(|_k, item| {
                        #[allow(clippy::redundant_closure_call)]
                        let span = $getter(item);
                        match span.source_id() {
                            Some(source_id) => &source_id.program_id() != program_id,
                            None => true,
                        }
                    });
                )*
            }
        }
    };
}

decl_engine_clear_program!(
    (variable_slab, |item: &VariableDeclaration| item.name.span()),
    (function_slab, |item: &FunctionDeclaration| item.name.span()),
    (trait_slab, |item: &TraitDeclaration| item.name.span()),
    (trait_fn_slab, |item: &TraitFn| item.name.span()),
    (trait_type_slab, |item: &TraitTypeDeclaration| item
        .name
        .span()),
    (impl_self_or_trait_slab, |item: &ImplSelfOrTrait| item
        .block_span
        .clone()),
    (struct_slab, |item: &StructDeclaration| item.name.span()),
    (storage_slab, |item: &StorageDeclaration| item.span.clone()),
    (abi_slab, |item: &AbiDeclaration| item.name.span()),
    (constant_slab, |item: &ConstantDeclaration| item.name.span()),
    (enum_slab, |item: &EnumDeclaration| item.name.span()),
    (type_alias_slab, |item: &TypeAliasDeclaration| item
        .name
        .span()),
);

macro_rules! decl_engine_clear_module {
    ($(($slab:ident, $getter:expr)),* $(,)?) => {
        impl ParsedDeclEngine {
            pub fn clear_module(&mut self, program_id: &SourceId) {
                $(
                    self.$slab.retain(|_k, item| {
                        #[allow(clippy::redundant_closure_call)]
                        let span = $getter(item);
                        match span.source_id() {
                            Some(src_id) => src_id != program_id,
                            None => true,
                        }
                    });
                )*
            }
        }
    };
}

decl_engine_clear_module!(
    (variable_slab, |item: &VariableDeclaration| item.name.span()),
    (function_slab, |item: &FunctionDeclaration| item.name.span()),
    (trait_slab, |item: &TraitDeclaration| item.name.span()),
    (trait_fn_slab, |item: &TraitFn| item.name.span()),
    (trait_type_slab, |item: &TraitTypeDeclaration| item
        .name
        .span()),
    (impl_self_or_trait_slab, |item: &ImplSelfOrTrait| item
        .block_span
        .clone()),
    (struct_slab, |item: &StructDeclaration| item.name.span()),
    (storage_slab, |item: &StorageDeclaration| item.span.clone()),
    (abi_slab, |item: &AbiDeclaration| item.name.span()),
    (constant_slab, |item: &ConstantDeclaration| item.name.span()),
    (enum_slab, |item: &EnumDeclaration| item.name.span()),
    (type_alias_slab, |item: &TypeAliasDeclaration| item
        .name
        .span()),
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
    pub fn get_impl_self_or_trait<I>(&self, index: &I) -> Arc<ImplSelfOrTrait>
    where
        ParsedDeclEngine: ParsedDeclEngineGet<I, ImplSelfOrTrait>,
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
    pub fn get_configurable<I>(&self, index: &I) -> Arc<ConfigurableDeclaration>
    where
        ParsedDeclEngine: ParsedDeclEngineGet<I, ConfigurableDeclaration>,
    {
        self.get(index)
    }

    /// Friendly helper method for calling the `get` method from the
    /// implementation of [ParsedDeclEngineGet] for [ParsedDeclEngine]
    ///
    /// Calling [ParsedDeclEngine][get] directly is equivalent to this method, but
    /// this method adds additional syntax that some users may find helpful.
    pub fn get_const_generic<I>(&self, index: &I) -> Arc<ConstGenericDeclaration>
    where
        ParsedDeclEngine: ParsedDeclEngineGet<I, ConstGenericDeclaration>,
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
    pub fn get_enum_variant<I>(&self, index: &I) -> Arc<EnumVariant>
    where
        ParsedDeclEngine: ParsedDeclEngineGet<I, EnumVariant>,
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

    pub fn pretty_print(&self) -> String {
        use std::fmt::Write;
        let mut s = String::new();
        let _ = write!(
            &mut s,
            "Function Count: {}",
            self.function_slab.values().len()
        );
        for f in self.function_slab.values() {
            let _ = write!(&mut s, "Function: {}", f.name);
            for node in f.body.contents.iter() {
                let _ = write!(&mut s, "    Node: {node:#?}");
            }
        }

        s
    }
}
