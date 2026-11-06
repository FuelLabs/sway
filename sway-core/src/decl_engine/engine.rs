use parking_lot::RwLock;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Write,
    sync::Arc,
};
use sway_utils::DeclEngineMetrics;

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
    const_generic_parsed_decl_id_map:
        RwLock<HashMap<DeclId<TyConstGenericDecl>, ParsedDeclId<ConstGenericDeclaration>>>,
    configurable_parsed_decl_id_map:
        RwLock<HashMap<DeclId<TyConfigurableDecl>, ParsedDeclId<ConfigurableDeclaration>>>,
    enum_parsed_decl_id_map: RwLock<HashMap<DeclId<TyEnumDecl>, ParsedDeclId<EnumDeclaration>>>,
    type_alias_parsed_decl_id_map:
        RwLock<HashMap<DeclId<TyTypeAliasDecl>, ParsedDeclId<TypeAliasDeclaration>>>,

    /// Maps a declaration of a [FunctionalDeclId] to the declarations it is derived from,
    /// called the **parents** of the declaration.
    ///
    /// The parent links trace function ids referenced in function applications
    /// back to the original declarations these ids are derived from.
    ///
    /// There are exactly two kinds of parent links:
    /// - A trait-interface dummy function (see [DeclEngine::insert_dummy_func])
    ///   is derived from the trait interface function ([ty::TyTraitFn]) it stands in for.
    /// - A modified copy of a declaration (see [DeclEngineInsert::insert_modified]),
    ///   e.g., a monomorphized version of a generic declaration, is derived from
    ///   the original declaration.
    parents: RwLock<HashMap<FunctionalDeclId, Vec<FunctionalDeclId>>>,

    /// The currently open slots: decl slots reserved via [DeclEngineReserve::reserve]
    /// and not yet frozen via [DeclEngineReserve::freeze]. In other words, the decls
    /// that are currently in their construction phase.
    ///
    /// For each open slot, the [SourceId] of the corresponding parsed declaration
    /// is stored, captured at the reservation time. A reserved and never filled
    /// slot has no content, and thus no span. The stored [SourceId] is the only
    /// way to attribute such a slot to a module or program, which is needed for
    /// reclaiming abandoned open slots in [DeclEngine::clear_module] and
    /// [DeclEngine::clear_program]. Open slots get abandoned if a compilation
    /// is canceled during the type checking, e.g., when the LSP server
    /// retriggers a compilation on a code change.
    ///
    /// The open slots sets exist only for the decl types that implement
    /// [DeclEngineReserve]. Note that the sets do not provide, nor undermine, the
    /// mutation guarantees of the reserve-fill-freeze protocol. Those are given
    /// at compile time by the [ReservedDeclId] tokens. The sets are a bookkeeping
    /// mechanism used for the abandoned slots reclamation during [DeclEngine::clear_module]
    /// or [DeclEngine::clear_program].
    function_open_slots: RwLock<HashMap<DeclId<TyFunctionDecl>, Option<SourceId>>>,
    impl_self_or_trait_open_slots: RwLock<HashMap<DeclId<TyImplSelfOrTrait>, Option<SourceId>>>,
    trait_open_slots: RwLock<HashMap<DeclId<TyTraitDecl>, Option<SourceId>>>,
    abi_open_slots: RwLock<HashMap<DeclId<TyAbiDecl>, Option<SourceId>>>,

    /// The forgotten slots: decl slots whose construction was deliberately given
    /// up by forgetting them via [DeclEngine::forget_open_slots_since].
    ///
    /// Slots get forgotten when a speculative type checking fails and its errors
    /// are deliberately swallowed. E.g., the auto-implementations of traits like
    /// `AbiEncode` or `Debug` are generated speculatively for every struct and
    /// enum, and a failure to type check a generated implementation only means
    /// that the type does not get that implementation. Such a failed type checking
    /// drops the [ReservedDeclId] tokens of the declarations it has reserved, and
    /// its open slots would otherwise remain open forever, because no error
    /// reaches the compilation's error handler.
    ///
    /// Same as for the open slots, the stored [SourceId] of the corresponding
    /// parsed declaration is the only way to attribute a never filled slot to
    /// a module or program. The forgotten slots are reclaimed, together with the
    /// abandoned open slots, during [DeclEngine::clear_module] or
    /// [DeclEngine::clear_program]. Until then, they stay allocated, which
    /// guarantees that their [DeclId]s are never reused. Any references to
    /// them, e.g., in the `implementing_type`s of the function declarations
    /// created during the failed speculative type checking, can never point to
    /// an unrelated declaration.
    function_forgotten_slots: RwLock<HashMap<DeclId<TyFunctionDecl>, Option<SourceId>>>,
    impl_self_or_trait_forgotten_slots:
        RwLock<HashMap<DeclId<TyImplSelfOrTrait>, Option<SourceId>>>,
    trait_forgotten_slots: RwLock<HashMap<DeclId<TyTraitDecl>, Option<SourceId>>>,
    abi_forgotten_slots: RwLock<HashMap<DeclId<TyAbiDecl>, Option<SourceId>>>,
}

/// A snapshot of the [DeclId]s of the decl slots that were open at the moment
/// of calling [DeclEngine::mark_open_slots]. The sole purpose of the mark is
/// to scope a speculative type checking, so that the open slots it leaves
/// behind can be forgotten via [DeclEngine::forget_open_slots_since].
struct OpenDeclSlotsMark {
    functions: HashSet<DeclId<TyFunctionDecl>>,
    impls: HashSet<DeclId<TyImplSelfOrTrait>>,
    traits: HashSet<DeclId<TyTraitDecl>>,
    abis: HashSet<DeclId<TyAbiDecl>>,
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
            const_generic_parsed_decl_id_map: RwLock::new(
                self.const_generic_parsed_decl_id_map.read().clone(),
            ),
            configurable_parsed_decl_id_map: RwLock::new(
                self.configurable_parsed_decl_id_map.read().clone(),
            ),
            enum_parsed_decl_id_map: RwLock::new(self.enum_parsed_decl_id_map.read().clone()),
            type_alias_parsed_decl_id_map: RwLock::new(
                self.type_alias_parsed_decl_id_map.read().clone(),
            ),
            parents: RwLock::new(self.parents.read().clone()),
            function_open_slots: RwLock::new(self.function_open_slots.read().clone()),
            impl_self_or_trait_open_slots: RwLock::new(
                self.impl_self_or_trait_open_slots.read().clone(),
            ),
            trait_open_slots: RwLock::new(self.trait_open_slots.read().clone()),
            abi_open_slots: RwLock::new(self.abi_open_slots.read().clone()),
            function_forgotten_slots: RwLock::new(self.function_forgotten_slots.read().clone()),
            impl_self_or_trait_forgotten_slots: RwLock::new(
                self.impl_self_or_trait_forgotten_slots.read().clone(),
            ),
            trait_forgotten_slots: RwLock::new(self.trait_forgotten_slots.read().clone()),
            abi_forgotten_slots: RwLock::new(self.abi_forgotten_slots.read().clone()),
        }
    }
}

pub trait DeclEngineGet<I, U> {
    fn get(&self, index: &I) -> Arc<U>;
    fn map<R>(&self, index: &I, f: impl FnOnce(&U) -> R) -> R;
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
    /// Inserts a typed declaration `decl` that corresponds to the parsed declaration
    /// `parsed_decl_it` into the [DeclEngine] for the first time.
    ///
    /// This method is meant to be called **only during the initial type checking**,
    /// when the typed declaration is created for the first time.
    fn insert(&self, decl: T, parsed_decl_id: ParsedDeclId<T::ParsedType>) -> DeclRef<DeclId<T>>;

    /// Inserts a typed declaration `modified_decl` that represents a modified version of
    /// an `original_decl`. E.g., during monomorphization, we get an original typed declaration
    /// from the [DeclEngine], modify it, and insert the `modified_decl` into the [DeclEngine].
    ///
    /// The `modified_decl` will have the same corresponding parsed declaration as the `original_decl`.
    ///
    /// Panics if the `original_decl` does not exist in the [DeclEngine]
    /// or if it doesn't have its corresponding parsed declaration.
    ///
    /// The only typed declarations that legitimately have no corresponding parsed
    /// declaration are the trait-interface dummy functions inserted via
    /// [DeclEngine::insert_dummy_func]. For those, [DeclEngineInsert] is implemented
    /// manually (see the implementation for [ty::TyFunctionDecl]).
    ///
    /// For [ty::TyFunctionDecl]s and [ty::TyTraitFn]s, the `original_decl` is automatically
    /// registered as the parent of the `modified_decl`. Hence, both types have manual
    /// implementations of [DeclEngineInsert].
    fn insert_modified(&self, modified_decl: T, original_decl: DeclId<T>) -> DeclRef<DeclId<T>>;
}

/// The **reserve-fill-freeze protocol** for inserting declarations whose construction
/// is inherently circular or multi-staged. E.g., impl blocks and their items, which
/// point at each other, or functions, whose bodies are type checked in multiple passes.
///
/// The protocol:
/// 1. [Self::reserve] allocates an empty decl slot and returns the [ReservedDeclId]
///    token. The [DeclId] of the slot is available via
///    [ReservedDeclId::id] and can immediately be embedded into other declarations.
///    Reading the declaration behind it panics (ICEs) until the first [Self::fill].
/// 2. [Self::fill] writes the declaration content into the slot. It can be called
///    multiple times, e.g., once per type checking pass, always under the same
///    [DeclId]. **The declaration is readable between the fills**.
/// 3. [Self::freeze] consumes the token and thereby ends the construction phase.
///    Since the token is neither [Clone] nor [Copy], after the freeze the
///    declaration is guaranteed, at compile time, to be immutable.
///
/// If the construction fails, e.g., because the type checking of the declaration
/// returns an error, the token is simply dropped and the slot remains open:
/// - In a regular compilation, the error fails the compilation, in which case
///   the open slots are irrelevant.
/// - In a speculative type checking whose errors are deliberately swallowed,
///   e.g., in the trait auto-implementations, the speculation boundary forgets
///   the slots it has left open. Speculations always run within the
///   [SpeculationHandler][crate::SpeculationHandler] API, which establishes
///   that boundary.
///
/// In both cases, the abandoned open slots are reclaimed during
/// [DeclEngine::clear_module] or [DeclEngine::clear_program].
///
/// This trait is implemented only for the declaration types that actually
/// need the reservation, and is the only way of modifying a
/// declaration contained in the [DeclEngine].
pub trait DeclEngineReserve<T>
where
    T: Named + Spanned + TyDeclParsedType,
{
    /// Reserves an empty decl slot and returns the token for filling
    /// and freezing it.
    ///
    /// The `parsed_decl_id` of the corresponding parsed declaration is registered
    /// immediately, same as in [DeclEngineInsert::insert]. The span of the parsed
    /// declaration attributes the reserved slot to a module, so that abandoned
    /// slots can be reclaimed in [DeclEngine::clear_module] and
    /// [DeclEngine::clear_program]. (A reserved and never filled slot has no
    /// content, and thus no span of its own.)
    fn reserve(
        &self,
        engines: &Engines,
        parsed_decl_id: ParsedDeclId<T::ParsedType>,
    ) -> ReservedDeclId<T>;

    /// Fills the reserved slot with the `decl` content. Can be called multiple
    /// times; every fill completely replaces the previous content, always under
    /// the same [DeclId].
    fn fill(&self, reserved: &ReservedDeclId<T>, decl: T);

    /// Ends the construction phase of the declaration in the reserved slot by
    /// consuming the token, and returns the [DeclRef] of the now immutable declaration.
    ///
    /// Panics if the slot was never filled.
    fn freeze(&self, reserved: ReservedDeclId<T>) -> DeclRef<DeclId<T>>;
}

pub trait DeclEngineIndex<T>: DeclEngineGet<DeclId<T>, T>
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
                parsed_decl_id: ParsedDeclId<<$decl as TyDeclParsedType>::ParsedType>,
            ) -> DeclRef<DeclId<$decl>> {
                let span = decl.span();
                let decl_name = decl.name().clone();
                let decl_id = DeclId::new(self.$slab.insert(decl));
                self.$parsed_slab.write().insert(decl_id, parsed_decl_id);
                DeclRef::new(decl_name, decl_id, span)
            }

            fn insert_modified(
                &self,
                modified_decl: $decl,
                original_decl: DeclId<$decl>,
            ) -> DeclRef<DeclId<$decl>> {
                self.insert(
                    modified_decl,
                    self.get_parsed_decl_id(&original_decl)
                        .expect("`original_decl` must have a corresponding parsed declaration"),
                )
            }
        }
    };
}

/// [ty::TyFunctionDecl] is intentionally not implemented via the `decl_engine_insert!`
/// macro, for two reasons:
/// - Unlike all other typed declarations, functions have one legitimate case of not
///   having a corresponding parsed declaration: the trait-interface dummy functions inserted
///   via [DeclEngine::insert_dummy_func]. `insert_modified` must therefore tolerate a missing
///   parsed declaration, but **only for such dummy functions**.
/// - `insert_modified` automatically registers the `original_decl` as the parent of the
///   `modified_decl`.
impl DeclEngineInsert<ty::TyFunctionDecl> for DeclEngine {
    fn insert(
        &self,
        decl: ty::TyFunctionDecl,
        parsed_decl_id: ParsedDeclId<<ty::TyFunctionDecl as TyDeclParsedType>::ParsedType>,
    ) -> DeclRef<DeclId<ty::TyFunctionDecl>> {
        let span = decl.span();
        let decl_name = decl.name().clone();
        let decl_id = DeclId::new(self.function_slab.insert(decl));
        self.function_parsed_decl_id_map
            .write()
            .insert(decl_id, parsed_decl_id);
        DeclRef::new(decl_name, decl_id, span)
    }

    fn insert_modified(
        &self,
        modified_decl: ty::TyFunctionDecl,
        original_decl: DeclId<ty::TyFunctionDecl>,
    ) -> DeclRef<DeclId<ty::TyFunctionDecl>> {
        let decl_ref = match self.get_parsed_decl_id(&original_decl) {
            // The `modified_decl` inherits the parsed declaration of the
            // `original_decl` it was cloned and modified from.
            Some(parsed_decl_id) => self.insert(modified_decl, parsed_decl_id),
            // The only functions that legitimately lack a parsed declaration are the
            // trait-interface dummy functions. If the `original_decl` had no parsed
            // declaration and was not a dummy function, that is a bug and the below
            // call will panic.
            None => self.insert_function_without_parsed_decl(modified_decl),
        };
        // The `modified_decl` is derived from the `original_decl`. Register the
        // `original_decl` as its parent.
        self.register_parent((*decl_ref.id()).into(), original_decl.into());
        decl_ref
    }
}

/// [ty::TyTraitFn] is intentionally not implemented via the `decl_engine_insert!`
/// macro, because, same as for [ty::TyFunctionDecl], `insert_modified` automatically
/// registers the `original_decl` as the parent of the `modified_decl`.
impl DeclEngineInsert<ty::TyTraitFn> for DeclEngine {
    fn insert(
        &self,
        decl: ty::TyTraitFn,
        parsed_decl_id: ParsedDeclId<<ty::TyTraitFn as TyDeclParsedType>::ParsedType>,
    ) -> DeclRef<DeclId<ty::TyTraitFn>> {
        let span = decl.span();
        let decl_name = decl.name().clone();
        let decl_id = DeclId::new(self.trait_fn_slab.insert(decl));
        self.trait_fn_parsed_decl_id_map
            .write()
            .insert(decl_id, parsed_decl_id);
        DeclRef::new(decl_name, decl_id, span)
    }

    fn insert_modified(
        &self,
        modified_decl: ty::TyTraitFn,
        original_decl: DeclId<ty::TyTraitFn>,
    ) -> DeclRef<DeclId<ty::TyTraitFn>> {
        let decl_ref = self.insert(
            modified_decl,
            self.get_parsed_decl_id(&original_decl)
                .expect("`original_decl` must have a corresponding parsed declaration"),
        );
        // The `modified_decl` is derived from the `original_decl`. Register the
        // `original_decl` as its parent.
        self.register_parent((*decl_ref.id()).into(), original_decl.into());
        decl_ref
    }
}

decl_engine_insert!(trait_slab, trait_parsed_decl_id_map, ty::TyTraitDecl);
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
    const_generic_parsed_decl_id_map,
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
decl_engine_parsed_decl_id!(const_generic_parsed_decl_id_map, ty::TyConstGenericDecl);
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
    const_generic_parsed_decl_id_map,
    ty::TyConstGenericDecl,
    Declaration::ConstGenericDeclaration
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

macro_rules! decl_engine_reserve {
    ($slab:ident, $parsed_slab:ident, $open_slots:ident, $decl:ty) => {
        impl DeclEngineReserve<$decl> for DeclEngine {
            fn reserve(
                &self,
                engines: &Engines,
                parsed_decl_id: ParsedDeclId<<$decl as TyDeclParsedType>::ParsedType>,
            ) -> ReservedDeclId<$decl> {
                let source_id = engines
                    .pe()
                    .get(&parsed_decl_id)
                    .span()
                    .source_id()
                    .copied();
                let decl_id = DeclId::new(self.$slab.reserve());
                self.$parsed_slab.write().insert(decl_id, parsed_decl_id);
                self.$open_slots.write().insert(decl_id, source_id);
                ReservedDeclId::new(decl_id)
            }

            fn fill(&self, reserved: &ReservedDeclId<$decl>, decl: $decl) {
                self.$slab.fill(reserved.id().inner(), decl);
            }

            fn freeze(&self, reserved: ReservedDeclId<$decl>) -> DeclRef<DeclId<$decl>> {
                let decl_id = reserved.id();
                assert!(
                    self.$slab.is_filled(decl_id.inner()),
                    "a reserved decl slot must be filled before it is frozen"
                );
                self.$open_slots.write().remove(&decl_id);
                let (name, span) = self
                    .$slab
                    .map(decl_id.inner(), |decl| (decl.name().clone(), decl.span()));
                DeclRef::new(name, decl_id, span)
            }
        }
    };
}
decl_engine_reserve!(
    function_slab,
    function_parsed_decl_id_map,
    function_open_slots,
    ty::TyFunctionDecl
);
decl_engine_reserve!(
    impl_self_or_trait_slab,
    impl_self_or_trait_parsed_decl_id_map,
    impl_self_or_trait_open_slots,
    ty::TyImplSelfOrTrait
);
decl_engine_reserve!(
    trait_slab,
    trait_parsed_decl_id_map,
    trait_open_slots,
    ty::TyTraitDecl
);
decl_engine_reserve!(
    abi_slab,
    abi_parsed_decl_id_map,
    abi_open_slots,
    ty::TyAbiDecl
);

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
                self.reclaim_abandoned_open_and_forgotten_slots(&|source_id: &SourceId| {
                    &source_id.program_id() == program_id
                });

                self.parents.write().retain(|key, _| {
                    match key {
                        FunctionalDeclId::TraitFn(decl_id) => {
                            self.get_trait_fn(decl_id).span().source_id().map_or(true, |src_id| &src_id.program_id() != program_id)
                        },
                        FunctionalDeclId::Function(decl_id) => {
                            self.get_function(decl_id).span().source_id().map_or(true, |src_id| &src_id.program_id() != program_id)
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
                self.reclaim_abandoned_open_and_forgotten_slots(&|id: &SourceId| id == source_id);

                self.parents.write().retain(|key, _| {
                    match key {
                        FunctionalDeclId::TraitFn(decl_id) => {
                            self.get_trait_fn(decl_id).span().source_id().map_or(true, |src_id| src_id != source_id)
                        },
                        FunctionalDeclId::Function(decl_id) => {
                            self.get_function(decl_id).span().source_id().map_or(true, |src_id| src_id != source_id)
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
    /// Returns true if there are decl slots that are currently in their
    /// construction phase (reserved and not yet frozen).
    ///
    /// After a successfully finished type checking of a program, there must
    /// be no open decl slots - every reservation must be frozen. Open slots
    /// can outlive a type checking only if the compilation fails or is
    /// canceled, e.g., when the LSP server retriggers a compilation on
    /// a code change. Such abandoned slots are reclaimed in
    /// [DeclEngine::clear_module] and [DeclEngine::clear_program].
    ///
    /// Note that the forgotten slots are not open slots. A failed speculative
    /// type checking whose errors are deliberately swallowed forgets the slots
    /// it has left open (see [SpeculationHandler][crate::SpeculationHandler]),
    /// and does not violate the above invariant.
    pub fn has_open_decl_slots(&self) -> bool {
        !(self.function_open_slots.read().is_empty()
            && self.impl_self_or_trait_open_slots.read().is_empty()
            && self.trait_open_slots.read().is_empty()
            && self.abi_open_slots.read().is_empty())
    }

    /// Runs `f` and forgets all the decl slots that `f` leaves open, by moving
    /// them from the open slots to the forgotten slots.
    ///
    /// This is the decl slots clean-up of a speculation boundary, and
    /// [SpeculationHandler::speculate][crate::SpeculationHandler::speculate]
    /// must be its only caller.
    /// Run speculations within the [SpeculationHandler][crate::SpeculationHandler]
    /// API - never call this method directly.
    ///
    /// Forgetting does not deallocate anything. The forgotten slots, and the
    /// garbage declarations created during the failed speculation, stay in the
    /// engine until they are reclaimed during [DeclEngine::clear_module] or
    /// [DeclEngine::clear_program], attributed to their module via their
    /// autogenerated [SourceId]s.
    pub(crate) fn forget_open_slots_during<T>(&self, f: impl FnOnce() -> T) -> T {
        let mark = self.mark_open_slots();
        let result = f();
        self.forget_open_slots_since(mark);
        result
    }

    /// Marks the currently open decl slots and returns the mark, to be used
    /// in a later call to [DeclEngine::forget_open_slots_since].
    fn mark_open_slots(&self) -> OpenDeclSlotsMark {
        OpenDeclSlotsMark {
            functions: self.function_open_slots.read().keys().copied().collect(),
            impls: self
                .impl_self_or_trait_open_slots
                .read()
                .keys()
                .copied()
                .collect(),
            traits: self.trait_open_slots.read().keys().copied().collect(),
            abis: self.abi_open_slots.read().keys().copied().collect(),
        }
    }

    /// Forgets all the decl slots opened since the `mark` was taken, by moving
    /// them from the open slots to the forgotten slots
    /// (see [DeclEngine::forget_open_slots_during]).
    fn forget_open_slots_since(&self, mark: OpenDeclSlotsMark) {
        macro_rules! forget {
            ($open_slots:ident, $forgotten_slots:ident, $marked:expr) => {{
                let mut open_slots = self.$open_slots.write();
                let forgotten: Vec<_> = open_slots
                    .keys()
                    .filter(|decl_id| !$marked.contains(decl_id))
                    .copied()
                    .collect();
                let mut forgotten_slots = self.$forgotten_slots.write();
                for decl_id in forgotten {
                    let source_id = open_slots.remove(&decl_id).flatten();
                    forgotten_slots.insert(decl_id, source_id);
                }
            }};
        }
        forget!(
            function_open_slots,
            function_forgotten_slots,
            mark.functions
        );
        forget!(
            impl_self_or_trait_open_slots,
            impl_self_or_trait_forgotten_slots,
            mark.impls
        );
        forget!(trait_open_slots, trait_forgotten_slots, mark.traits);
        forget!(abi_open_slots, abi_forgotten_slots, mark.abis);
    }

    /// Reclaims the abandoned open decl slots and the forgotten decl slots
    /// (see [DeclEngine::forget_open_slots_since]) attributable to a cleared
    /// module or program. Reclaiming removes their open or forgotten slots
    /// entries. If a slot was never filled (and thus cannot be seen by the
    /// span-based clearing of the slabs, because it has no content), returns
    /// the slot to the free list and removes its parsed decl id mapping.
    ///
    /// Note that this method must only be called in between compilations, when
    /// the [ReservedDeclId] tokens owning the open slots are guaranteed to be
    /// dropped (the compilation that created them is finished or canceled).
    /// This is the same contract under which [DeclEngine::clear_module] and
    /// [DeclEngine::clear_program], which call this method, operate.
    fn reclaim_abandoned_open_and_forgotten_slots(
        &mut self,
        is_cleared: &impl Fn(&SourceId) -> bool,
    ) {
        macro_rules! reclaim {
            ($slab:ident, $parsed_slab:ident, $slots:ident) => {{
                let mut slots = self.$slots.write();
                let abandoned: Vec<_> = slots
                    .iter()
                    .filter(|(_, source_id)| source_id.is_some_and(|id| is_cleared(&id)))
                    .map(|(decl_id, _)| *decl_id)
                    .collect();
                for decl_id in abandoned {
                    slots.remove(&decl_id);
                    if !self.$slab.is_filled(decl_id.inner()) {
                        self.$slab.release_reserved(decl_id.inner());
                        self.$parsed_slab.write().remove(&decl_id);
                    }
                }
            }};
        }
        reclaim!(
            function_slab,
            function_parsed_decl_id_map,
            function_open_slots
        );
        reclaim!(
            impl_self_or_trait_slab,
            impl_self_or_trait_parsed_decl_id_map,
            impl_self_or_trait_open_slots
        );
        reclaim!(trait_slab, trait_parsed_decl_id_map, trait_open_slots);
        reclaim!(abi_slab, abi_parsed_decl_id_map, abi_open_slots);
        reclaim!(
            function_slab,
            function_parsed_decl_id_map,
            function_forgotten_slots
        );
        reclaim!(
            impl_self_or_trait_slab,
            impl_self_or_trait_parsed_decl_id_map,
            impl_self_or_trait_forgotten_slots
        );
        reclaim!(trait_slab, trait_parsed_decl_id_map, trait_forgotten_slots);
        reclaim!(abi_slab, abi_parsed_decl_id_map, abi_forgotten_slots);
    }

    /// Given a [DeclId] `index`, finds all the parents of `index` and all the
    /// recursive parents of those parents, and so on. Does not perform
    /// duplicated computation. If a parent of a [DeclId] has already been
    /// found, we do not find it again.
    ///
    /// See the documentation of the [DeclEngine::parents] field for the
    /// explanation of parents.
    #[allow(clippy::map_entry)]
    pub(crate) fn find_all_parents<'a, T>(
        &self,
        engines: &Engines,
        index: &'a T,
    ) -> Vec<FunctionalDeclId>
    where
        FunctionalDeclId: From<&'a T>,
    {
        let index: FunctionalDeclId = FunctionalDeclId::from(index);
        let parents = self.parents.read();
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
                        ) => self.get(x_id).eq(
                            &self.get(curr_parent_id),
                            &PartialEqWithEnginesContext::new(engines),
                        ),
                        (
                            FunctionalDeclId::Function(x_id),
                            FunctionalDeclId::Function(curr_parent_id),
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

    /// Registers the `parent` as a parent of the declaration `index`.
    /// See [DeclEngine::parents] for the explanation of parents.
    ///
    /// This method is deliberately private to the [DeclEngine]. Parents are
    /// registered automatically, as a part of inserting declarations via
    /// [DeclEngineInsert::insert_modified] and [DeclEngine::insert_dummy_func].
    fn register_parent(&self, index: FunctionalDeclId, parent: FunctionalDeclId) {
        let mut parents = self.parents.write();
        parents
            .entry(index)
            .and_modify(|e| e.push(parent.clone()))
            .or_insert_with(|| vec![parent]);
    }

    /// Inserts a trait-interface **dummy function** into the [DeclEngine]
    /// and registers the trait interface function `interface_fn`, from which
    /// the dummy function is created, as its parent.
    ///
    /// This is a bit of a maverick compared to the regular [DeclEngineInsert::insert].
    /// It deliberately inserts a [ty::TyFunctionDecl] **without an associated parsed
    /// declaration**.
    ///
    /// Dummy functions are placeholders created from trait interface methods
    /// (see [ty::TyTraitFn::to_dummy_func]). They allow the trait's
    /// provided methods to refer to interface methods before an actual implementation
    /// exists. Their parsed origin is a [TraitFn] (i.e., a parsed trait function), not a
    /// [FunctionDeclaration], so there is no [FunctionDeclaration] to associate them with.
    ///
    /// This is the only semantically valid case of a [ty::TyFunctionDecl] having no
    /// corresponding parsed declaration. Everything else must go through
    /// [DeclEngineInsert::insert] or [DeclEngineInsert::insert_modified], which guarantee
    /// (and, for `insert_modified`, assert) the presence of a parsed declaration.
    ///
    /// Registering the `interface_fn` as the parent is crucial. It allows the
    /// references to the dummy function, e.g., calls to it within trait's provided
    /// methods, to be traced back to the interface function, and thus replaced by
    /// its actual implementation, once the implementation is known.
    ///
    /// Panics if the `decl` is not a trait-interface dummy function.
    pub fn insert_dummy_func(
        &self,
        decl: ty::TyFunctionDecl,
        interface_fn: DeclId<ty::TyTraitFn>,
    ) -> DeclRef<DeclId<ty::TyFunctionDecl>> {
        let decl_ref = self.insert_function_without_parsed_decl(decl);
        // The dummy function is derived from the trait interface function.
        // Register the `interface_fn` as its parent.
        self.register_parent((*decl_ref.id()).into(), interface_fn.into());
        decl_ref
    }

    /// Inserts a [ty::TyFunctionDecl] **without an associated parsed declaration**
    /// and **without registering any parents**.
    ///
    /// The only functions that legitimately lack a parsed declaration are the
    /// trait-interface dummy functions (see [DeclEngine::insert_dummy_func]),
    /// either the initially created ones, or their modified versions inserted
    /// via [DeclEngineInsert::insert_modified].
    ///
    /// Panics if the `decl` is not a trait-interface dummy function.
    fn insert_function_without_parsed_decl(
        &self,
        decl: ty::TyFunctionDecl,
    ) -> DeclRef<DeclId<ty::TyFunctionDecl>> {
        assert!(
            decl.is_trait_method_dummy,
            "only trait-interface dummy functions can be inserted without a corresponding parsed declaration"
        );
        let span = decl.span();
        let decl_name = decl.name().clone();
        let decl_id = DeclId::new(self.function_slab.insert(decl));
        DeclRef::new(decl_name, decl_id, span)
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

    pub fn metrics(&self) -> DeclEngineMetrics {
        DeclEngineMetrics {
            slabs: vec![
                self.function_slab.metrics("function_slab".into()),
                self.struct_slab.metrics("struct_slab".into()),
                self.enum_slab.metrics("enum_slab".into()),
                self.trait_slab.metrics("trait_slab".into()),
                self.trait_fn_slab.metrics("trait_fn_slab".into()),
                self.trait_type_slab.metrics("trait_type_slab".into()),
                self.impl_self_or_trait_slab
                    .metrics("impl_self_or_trait_slab".into()),
                self.storage_slab.metrics("storage_slab".into()),
                self.abi_slab.metrics("abi_slab".into()),
                self.constant_slab.metrics("constant_slab".into()),
                self.configurable_slab.metrics("configurable_slab".into()),
                self.const_generics_slab
                    .metrics("const_generics_slab".into()),
                self.type_alias_slab.metrics("type_alias_slab".into()),
            ],
        }
    }
}
