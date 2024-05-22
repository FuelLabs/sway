use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap},
    fmt,
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
};

use hashbrown::HashSet;
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{BaseIdent, Ident, Span, Spanned};

use crate::{
    decl_engine::{DeclEngineGet, DeclEngineInsert},
    engine_threading::*,
    language::{
        parsed::ImplItem,
        ty::{self, TyImplItem, TyTraitItem},
        CallPath,
    },
    type_system::{SubstTypes, TypeId},
    TraitConstraint, TypeArgument, TypeEngine, TypeInfo, TypeSubstMap, UnifyCheck,
};

use super::TryInsertingTraitImplOnFailure;

#[derive(Clone, Debug)]
struct TraitSuffix {
    name: Ident,
    args: Vec<TypeArgument>,
}
impl PartialEqWithEngines for TraitSuffix {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name == other.name && self.args.eq(&other.args, ctx)
    }
}
impl OrdWithEngines for TraitSuffix {
    fn cmp(&self, other: &Self, ctx: &OrdWithEnginesContext) -> std::cmp::Ordering {
        self.name
            .cmp(&other.name)
            .then_with(|| self.args.cmp(&other.args, ctx))
    }
}

impl DisplayWithEngines for TraitSuffix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        let res = write!(f, "{}", self.name.as_str());
        if !self.args.is_empty() {
            write!(
                f,
                "<{}>",
                self.args
                    .iter()
                    .map(|i| engines.help_out(i.type_id).to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            res
        }
    }
}

impl DebugWithEngines for TraitSuffix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(f, "{}", engines.help_out(self))
    }
}

type TraitName = Arc<CallPath<TraitSuffix>>;

#[derive(Clone, Debug)]
struct TraitKey {
    name: TraitName,
    type_id: TypeId,
    trait_decl_span: Option<Span>,
}

impl OrdWithEngines for TraitKey {
    fn cmp(&self, other: &Self, ctx: &OrdWithEnginesContext) -> std::cmp::Ordering {
        self.name
            .cmp(&other.name, ctx)
            .then_with(|| self.type_id.cmp(&other.type_id))
    }
}

#[derive(Clone, Debug)]
pub enum ResolvedTraitImplItem {
    Parsed(ImplItem),
    Typed(TyImplItem),
}

impl ResolvedTraitImplItem {
    fn expect_typed(self) -> TyImplItem {
        match self {
            ResolvedTraitImplItem::Parsed(_) => panic!(),
            ResolvedTraitImplItem::Typed(ty) => ty,
        }
    }

    pub fn span(&self, engines: &Engines) -> Span {
        match self {
            ResolvedTraitImplItem::Parsed(item) => item.span(engines),
            ResolvedTraitImplItem::Typed(item) => item.span(),
        }
    }
}

/// Map of name to [ResolvedTraitImplItem](ResolvedTraitImplItem)
type TraitItems = im::HashMap<String, ResolvedTraitImplItem>;

#[derive(Clone, Debug)]
struct TraitValue {
    trait_items: TraitItems,
    /// The span of the entire impl block.
    impl_span: Span,
}

#[derive(Clone, Debug)]
struct TraitEntry {
    key: TraitKey,
    value: TraitValue,
}

/// Map of trait name and type to [TraitItems].
type TraitImpls = Vec<TraitEntry>;

/// Map holding trait implementations for types.
///
/// Note: "impl self" blocks are considered traits and are stored in the
/// [TraitMap].
#[derive(Clone, Debug, Default)]
pub(crate) struct TraitMap {
    trait_impls: TraitImpls,
    satisfied_cache: hashbrown::HashSet<u64>,
}

pub(crate) enum IsImplSelf {
    Yes,
    No,
}

pub(crate) enum IsExtendingExistingImpl {
    Yes,
    No,
}

impl TraitMap {
    /// Given a [TraitName] `trait_name`, [TypeId] `type_id`, and list of
    /// [TyImplItem](ty::TyImplItem) `items`, inserts
    /// `items` into the [TraitMap] with the key `(trait_name, type_id)`.
    ///
    /// This method is as conscious as possible of existing entries in the
    /// [TraitMap], and tries to append `items` to an existing list of
    /// declarations for the key `(trait_name, type_id)` whenever possible.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn insert(
        &mut self,
        handler: &Handler,
        trait_name: CallPath,
        trait_type_args: Vec<TypeArgument>,
        type_id: TypeId,
        items: &[ResolvedTraitImplItem],
        impl_span: &Span,
        trait_decl_span: Option<Span>,
        is_impl_self: IsImplSelf,
        is_extending_existing_impl: IsExtendingExistingImpl,
        engines: &Engines,
    ) -> Result<(), ErrorEmitted> {
        handler.scope(|handler| {
            let mut trait_items: TraitItems = im::HashMap::new();
            for item in items.iter() {
                match item {
                    ResolvedTraitImplItem::Parsed(_) => todo!(),
                    ResolvedTraitImplItem::Typed(ty_item) => match ty_item {
                        TyImplItem::Fn(decl_ref) => {
                            if trait_items
                                .insert(decl_ref.name().clone().to_string(), item.clone())
                                .is_some()
                            {
                                // duplicate method name
                                handler.emit_err(CompileError::MultipleDefinitionsOfName {
                                    name: decl_ref.name().clone(),
                                    span: decl_ref.span(),
                                });
                            }
                        }
                        TyImplItem::Constant(decl_ref) => {
                            trait_items.insert(decl_ref.name().to_string(), item.clone());
                        }
                        TyImplItem::Type(decl_ref) => {
                            trait_items.insert(decl_ref.name().to_string(), item.clone());
                        }
                    },
                }
            }

            // check to see if adding this trait will produce a conflicting definition
            for TraitEntry {
                key:
                    TraitKey {
                        name: map_trait_name,
                        type_id: map_type_id,
                        trait_decl_span: _,
                    },
                value:
                    TraitValue {
                        trait_items: map_trait_items,
                        impl_span: existing_impl_span,
                    },
            } in self.trait_impls.iter()
            {
                let CallPath {
                    suffix:
                        TraitSuffix {
                            name: map_trait_name_suffix,
                            args: map_trait_type_args,
                        },
                    ..
                } = &*map_trait_name.clone();

                let unify_checker = UnifyCheck::non_generic_constraint_subset(engines);

                // Types are subset if the `type_id` that we want to insert can unify with the
                // existing `map_type_id`. In addition we need to additionally check for the case of
                // `&mut <type>` and `&<type>`.
                let types_are_subset = unify_checker.check(type_id, *map_type_id)
                    && is_unified_type_subset(engines.te(), type_id, *map_type_id);

                /// `left` can unify into `right`. Additionally we need to check subset condition in case of
                /// [TypeInfo::Ref] types.  Although `&mut <type>` can unify with `&<type>`
                /// when it comes to trait and self impls, we considered them to be different types.
                /// E.g., we can have `impl Foo for &T` and at the same time `impl Foo for &mut T`.
                /// Or in general, `impl Foo for & &mut .. &T` is different type then, e.g., `impl Foo for &mut & .. &mut T`.
                fn is_unified_type_subset(
                    type_engine: &TypeEngine,
                    mut left: TypeId,
                    mut right: TypeId,
                ) -> bool {
                    // The loop cannot be endless, because at the end we must hit a referenced type which is not
                    // a reference.
                    loop {
                        let left_ty_info = &*type_engine.get_unaliased(left);
                        let right_ty_info = &*type_engine.get_unaliased(right);
                        match (left_ty_info, right_ty_info) {
                            (
                                TypeInfo::Ref {
                                    to_mutable_value: l_to_mut,
                                    ..
                                },
                                TypeInfo::Ref {
                                    to_mutable_value: r_to_mut,
                                    ..
                                },
                            ) if *l_to_mut != *r_to_mut => return false, // Different mutability means not subset.
                            (
                                TypeInfo::Ref {
                                    referenced_type: l_ty,
                                    ..
                                },
                                TypeInfo::Ref {
                                    referenced_type: r_ty,
                                    ..
                                },
                            ) => {
                                left = l_ty.type_id;
                                right = r_ty.type_id;
                            }
                            _ => return true,
                        }
                    }
                }

                let mut traits_are_subset = true;
                if *map_trait_name_suffix != trait_name.suffix
                    || map_trait_type_args.len() != trait_type_args.len()
                {
                    traits_are_subset = false;
                } else {
                    for (map_arg_type, arg_type) in
                        map_trait_type_args.iter().zip(trait_type_args.iter())
                    {
                        if !unify_checker.check(arg_type.type_id, map_arg_type.type_id) {
                            traits_are_subset = false;
                        }
                    }
                }

                if matches!(is_extending_existing_impl, IsExtendingExistingImpl::No)
                    && types_are_subset
                    && traits_are_subset
                    && matches!(is_impl_self, IsImplSelf::No)
                {
                    let trait_name_str = format!(
                        "{}{}",
                        trait_name,
                        if trait_type_args.is_empty() {
                            String::new()
                        } else {
                            format!(
                                "<{}>",
                                trait_type_args
                                    .iter()
                                    .map(|type_arg| engines.help_out(type_arg).to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )
                        }
                    );
                    handler.emit_err(CompileError::ConflictingImplsForTraitAndType {
                        trait_name: trait_name_str,
                        type_implementing_for: engines.help_out(type_id).to_string(),
                        existing_impl_span: existing_impl_span.clone(),
                        second_impl_span: impl_span.clone(),
                    });
                } else if types_are_subset
                    && (traits_are_subset || matches!(is_impl_self, IsImplSelf::Yes))
                {
                    for (name, item) in trait_items.iter() {
                        match item {
                            ResolvedTraitImplItem::Parsed(_item) => todo!(),
                            ResolvedTraitImplItem::Typed(item) => match item {
                                ty::TyTraitItem::Fn(decl_ref) => {
                                    if map_trait_items.get(name).is_some() {
                                        handler.emit_err(
                                            CompileError::DuplicateDeclDefinedForType {
                                                decl_kind: "method".into(),
                                                decl_name: decl_ref.name().to_string(),
                                                type_implementing_for: engines
                                                    .help_out(type_id)
                                                    .to_string(),
                                                span: decl_ref.name().span(),
                                            },
                                        );
                                    }
                                }
                                ty::TyTraitItem::Constant(decl_ref) => {
                                    if map_trait_items.get(name).is_some() {
                                        handler.emit_err(
                                            CompileError::DuplicateDeclDefinedForType {
                                                decl_kind: "constant".into(),
                                                decl_name: decl_ref.name().to_string(),
                                                type_implementing_for: engines
                                                    .help_out(type_id)
                                                    .to_string(),
                                                span: decl_ref.name().span(),
                                            },
                                        );
                                    }
                                }
                                ty::TyTraitItem::Type(decl_ref) => {
                                    if map_trait_items.get(name).is_some() {
                                        handler.emit_err(
                                            CompileError::DuplicateDeclDefinedForType {
                                                decl_kind: "type".into(),
                                                decl_name: decl_ref.name().to_string(),
                                                type_implementing_for: engines
                                                    .help_out(type_id)
                                                    .to_string(),
                                                span: decl_ref.name().span(),
                                            },
                                        );
                                    }
                                }
                            },
                        }
                    }
                }
            }
            let trait_name: TraitName = Arc::new(CallPath {
                prefixes: trait_name.prefixes,
                suffix: TraitSuffix {
                    name: trait_name.suffix,
                    args: trait_type_args,
                },
                is_absolute: trait_name.is_absolute,
            });

            // even if there is a conflicting definition, add the trait anyway
            self.insert_inner(
                trait_name,
                impl_span.clone(),
                trait_decl_span,
                type_id,
                trait_items,
                engines,
            );

            Ok(())
        })
    }

    fn insert_inner(
        &mut self,
        trait_name: TraitName,
        impl_span: Span,
        trait_decl_span: Option<Span>,
        type_id: TypeId,
        trait_methods: TraitItems,
        engines: &Engines,
    ) {
        let key = TraitKey {
            name: trait_name,
            type_id,
            trait_decl_span,
        };
        let value = TraitValue {
            trait_items: trait_methods,
            impl_span,
        };
        let entry = TraitEntry { key, value };
        let trait_impls: TraitImpls = vec![entry];
        let trait_map = TraitMap {
            trait_impls,
            satisfied_cache: HashSet::default(),
        };

        self.extend(trait_map, engines);
    }

    /// Given a [TypeId] `type_id`, retrieves entries in the [TraitMap] `self`
    /// for which `type_id` is a subset and re-inserts them under `type_id`.
    ///
    /// Here is an example of what this means. Imagine we have this Sway code:
    ///
    /// ```ignore
    /// struct Data<T, F> {
    ///     first: T,
    ///     second: F,
    /// }
    ///
    /// impl<T, F> Data<T, F> {
    ///     fn get_first(self) -> T {
    ///         self.first
    ///     }
    ///
    ///     fn get_second(self) -> F {
    ///         self.second
    ///     }
    /// }
    ///
    /// impl<T> Data<T, T> {
    ///     fn switch(ref mut self) {
    ///         let first = self.first;
    ///         self.first = self.second;
    ///         self.second = first;
    ///     }
    /// }
    ///
    /// impl Data<u8, u8> {
    ///     fn add_u8(ref mut self, input: u8) {
    ///         self.first += input;
    ///         self.second += input;
    ///     }
    /// }
    ///
    /// impl Data<bool, bool> {
    ///     fn inner_and(self) -> bool {
    ///         self.first && self.second
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let mut foo = Data {
    ///         first: 1u8,
    ///         second: 2u8,
    ///     };
    ///
    ///     let a = foo.get_first();
    ///     let b = foo.get_second();
    ///     foo.switch();
    ///     let c = foo.add_u8(3u8);
    ///     let d = foo.inner_and();    // fails to compile
    ///
    ///     let mut bar = Data {
    ///         first: true,
    ///         second: false,
    ///     };
    ///
    ///     let e = bar.get_first();
    ///     let f = bar.get_second();
    ///     bar.switch();
    ///     let g = bar.add_u8(3u8);    // fails to compile
    ///     let h = bar.inner_and();
    ///
    ///     let mut baz = Data {
    ///         first: 1u8,
    ///         second: false,
    ///     };
    ///
    ///     let i = baz.get_first();
    ///     let j = baz.get_second();
    ///     baz.switch();               // fails to compile
    ///     let k = baz.add_u8(3u8);    // fails to compile
    ///     let l = baz.inner_and();    // fails to compile
    /// }
    /// ```
    ///
    /// When we first create the type of `Data<u8, u8>` when we declare the
    /// variable `foo`, we need some way of gathering all of the applicable
    /// traits that have been implemented for `Data<u8, u8>`, even if they were
    /// not implemented for `Data<u8, u8>` directly. That's why we look for
    /// entries in the [TraitMap] `self` for which `type_id` is a subset and
    /// re-insert them under `type_id`. Moreover, the impl block for
    /// `Data<T, T>` needs to be able to call methods that are defined in the
    /// impl block of `Data<T, F>`
    pub(crate) fn insert_for_type(&mut self, engines: &Engines, type_id: TypeId) {
        self.extend(self.filter_by_type(type_id, engines), engines);
    }

    /// Given [TraitMap]s `self` and `other`, extend `self` with `other`,
    /// extending existing entries when possible.
    pub(crate) fn extend(&mut self, other: TraitMap, engines: &Engines) {
        for oe in other.trait_impls.into_iter() {
            let pos = self
                .trait_impls
                .binary_search_by(|se| se.key.cmp(&oe.key, &OrdWithEnginesContext::new(engines)));

            match pos {
                Ok(pos) => self.trait_impls[pos]
                    .value
                    .trait_items
                    .extend(oe.value.trait_items),
                Err(pos) => self.trait_impls.insert(pos, oe),
            }
        }
    }

    /// Filters the entries in `self` and return a new [TraitMap] with all of
    /// the entries from `self` that implement a trait from the declaration with that span.
    pub(crate) fn filter_by_trait_decl_span(&self, trait_decl_span: Span) -> TraitMap {
        let mut trait_map = TraitMap::default();
        for entry in self.trait_impls.iter() {
            if entry
                .key
                .trait_decl_span
                .as_ref()
                .map_or(false, |span| span == &trait_decl_span)
            {
                trait_map.trait_impls.push(entry.clone());
            }
        }
        trait_map
    }

    /// Filters the entries in `self` with the given [TypeId] `type_id` and
    /// return a new [TraitMap] with all of the entries from `self` for which
    /// `type_id` is a subtype. Additionally, the new [TraitMap] contains the
    /// entries for the inner types of `self`.
    ///
    /// An "inner type" of `self` is one that is contained within `self`, but
    /// not including `self`. So the types of the fields of a struct would be
    /// inner types, for instance.
    ///
    /// The new [TraitMap] must contain entries for the inner types of `self`
    /// because users will want to chain field access's and method calls.
    /// Here is some example Sway code to demonstrate this:
    ///
    /// `data.sw`:
    /// ```ignore
    /// library;
    ///
    /// enum MyResult<T, E> {
    ///     Ok: T,
    ///     Err: E,
    /// }
    ///
    /// impl<T, E> MyResult<T, E> {
    ///     fn is_ok(self) -> bool {
    ///         match self {
    ///             MyResult::Ok(_) => true,
    ///             _ => false,
    ///         }
    ///     }
    /// }
    ///
    /// pub struct Data<T> {
    ///     value: MyResult<T, str[10]>,
    /// }
    ///
    /// impl<T> Data<T> {
    ///     fn new(value: T) -> Data<T> {
    ///         Data {
    ///             value: MyResult::Ok(value)
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// `main.sw`:
    /// ```ignore
    /// script;
    ///
    /// mod data;
    ///
    /// use data::Data;
    ///
    /// fn main() {
    ///     let foo = Data::new(true);
    ///     let bar = foo.value.is_ok();
    /// }
    /// ```
    ///
    /// In this example, we need to be able to find the definition of the
    /// `is_ok` method for the correct type, but we need to do that without
    /// requiring the user to import the whole `MyResult<T, E>` enum. Because if
    /// this was required, this would make users make large portions of their
    /// libraries public with `pub`. Moreover, we wouldn't need to import the
    /// whole `MyResult<T, E>` enum anyway, because the only type that we are
    /// seeing in `main.sw` is `MyResult<bool, str[10]>`!
    ///
    /// When an entry is found from `self` with type `type_id'` for which
    /// `type_id` is a subtype, we take the methods defined upon `type_id'` and
    /// translate them to be defined upon `type_id`.
    ///
    /// Here is an example of what this looks like. Take this Sway code:
    ///
    /// ```ignore
    /// impl<T, F> Data<T, F> {
    ///     fn get_first(self) -> T {
    ///         self.first
    ///     }
    ///
    ///     fn get_second(self) -> F {
    ///         self.second
    ///     }
    /// }
    ///
    /// impl<T> Data<T, T> {
    ///     fn switch(ref mut self) {
    ///         let first = self.first;
    ///         self.first = self.second;
    ///         self.second = first;
    ///     }
    /// }
    /// ```
    ///
    /// If we were to list all of the methods by hand defined for `Data<T, T>`,
    /// these would be `get_first()`, `get_second()`, and `switch()`. But if we
    /// were to list all of the methods by hand for `Data<T, F>`, these would
    /// just include `get_first()` and `get_second()`. So, for any given
    /// [TraitMap], in order to find all of the methods defined for a `type_id`,
    /// we must iterate through the [TraitMap] and extract all methods that are
    /// defined upon any type for which `type_id` is a subset.
    ///
    /// Once those methods are identified, we need to translate them to be
    /// defined upon `type_id`. Imagine that `type_id` is `Data<T, T>`, when
    /// we iterate on `self` we find `Data<T, F>: get_first(self) -> T`,
    /// `Data<T, F>: get_second(self) -> F`. Once we translate these methods, we
    /// have `Data<T, T>: get_first(self) -> T` and
    /// `Data<T, T>: get_second(self) -> T`, and we can create a new [TraitMap]
    /// with those entries for `Data<T, T>`.
    pub(crate) fn filter_by_type(&self, type_id: TypeId, engines: &Engines) -> TraitMap {
        let unify_checker = UnifyCheck::constraint_subset(engines);

        // a curried version of the decider protocol to use in the helper functions
        let decider = |left: TypeId, right: TypeId| unify_checker.check(left, right);
        let mut all_types = type_id.extract_inner_types(engines);
        all_types.insert(type_id);
        let all_types = all_types.into_iter().collect::<Vec<_>>();
        self.filter_by_type_inner(engines, all_types, decider)
    }

    /// Filters the entries in `self` with the given [TypeId] `type_id` and
    /// return a new [TraitMap] with all of the entries from `self` for which
    /// `type_id` is a subtype or a supertype. Additionally, the new [TraitMap]
    /// contains the entries for the inner types of `self`.
    ///
    /// This is used for handling the case in which we need to import an impl
    /// block from another module, and the type that that impl block is defined
    /// for is of the type that we are importing, but in a more concrete form.
    ///
    /// Here is some example Sway code that we should expect to compile:
    ///
    /// `my_double.sw`:
    /// ```ignore
    /// library;
    ///
    /// pub trait MyDouble<T> {
    ///     fn my_double(self, input: T) -> T;
    /// }
    /// ```
    ///
    /// `my_point.sw`:
    /// ```ignore
    /// library;
    ///
    /// use ::my_double::MyDouble;
    ///
    /// pub struct MyPoint<T> {
    ///     x: T,
    ///     y: T,
    /// }
    ///
    /// impl MyDouble<u64> for MyPoint<u64> {
    ///     fn my_double(self, value: u64) -> u64 {
    ///         (self.x*2) + (self.y*2) + (value*2)
    ///     }
    /// }
    /// ```
    ///
    /// `main.sw`:
    /// ```ignore
    /// script;
    ///
    /// mod my_double;
    /// mod my_point;
    ///
    /// use my_point::MyPoint;
    ///
    /// fn main() -> u64 {
    ///     let foo = MyPoint {
    ///         x: 10u64,
    ///         y: 10u64,
    ///     };
    ///     foo.my_double(100)
    /// }
    /// ```
    ///
    /// We need to be able to import the trait defined upon `MyPoint<u64>` just
    /// from seeing `use ::my_double::MyDouble;`.
    pub(crate) fn filter_by_type_item_import(
        &self,
        type_id: TypeId,
        engines: &Engines,
    ) -> TraitMap {
        let unify_checker = UnifyCheck::constraint_subset(engines);
        let unify_checker_for_item_import = UnifyCheck::non_generic_constraint_subset(engines);

        // a curried version of the decider protocol to use in the helper functions
        let decider = |left: TypeId, right: TypeId| {
            unify_checker.check(left, right) || unify_checker_for_item_import.check(right, left)
        };
        let mut trait_map = self.filter_by_type_inner(engines, vec![type_id], decider);
        let all_types = type_id
            .extract_inner_types(engines)
            .into_iter()
            .collect::<Vec<_>>();
        // a curried version of the decider protocol to use in the helper functions
        let decider2 = |left: TypeId, right: TypeId| unify_checker.check(left, right);

        trait_map.extend(
            self.filter_by_type_inner(engines, all_types, decider2),
            engines,
        );
        trait_map
    }

    fn filter_by_type_inner(
        &self,
        engines: &Engines,
        mut all_types: Vec<TypeId>,
        decider: impl Fn(TypeId, TypeId) -> bool,
    ) -> TraitMap {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        let mut trait_map = TraitMap::default();
        for TraitEntry {
            key:
                TraitKey {
                    name: map_trait_name,
                    type_id: map_type_id,
                    trait_decl_span: map_trait_decl_span,
                },
            value:
                TraitValue {
                    trait_items: map_trait_items,
                    impl_span,
                },
        } in self.trait_impls.iter()
        {
            for type_id in all_types.iter_mut() {
                let type_info = type_engine.get(*type_id);
                if !type_info.can_change(decl_engine) && *type_id == *map_type_id {
                    trait_map.insert_inner(
                        map_trait_name.clone(),
                        impl_span.clone(),
                        map_trait_decl_span.clone(),
                        *type_id,
                        map_trait_items.clone(),
                        engines,
                    );
                } else if decider(*type_id, *map_type_id) {
                    let mut insertable = true;
                    if let TypeInfo::UnknownGeneric {
                        is_from_type_parameter,
                        ..
                    } = *engines.te().get(*map_type_id)
                    {
                        insertable = !is_from_type_parameter
                            || matches!(
                                *engines.te().get(*type_id),
                                TypeInfo::UnknownGeneric { .. }
                            );
                    }
                    let type_mapping = TypeSubstMap::from_superset_and_subset(
                        type_engine,
                        decl_engine,
                        *map_type_id,
                        *type_id,
                    );
                    type_id.subst(&type_mapping, engines);
                    let trait_items: TraitItems = map_trait_items
                        .clone()
                        .into_iter()
                        .filter_map(|(name, item)| match &item {
                            ResolvedTraitImplItem::Parsed(_item) => todo!(),
                            ResolvedTraitImplItem::Typed(item) => match item {
                                ty::TyTraitItem::Fn(decl_ref) => {
                                    let mut decl = (*decl_engine.get(decl_ref.id())).clone();
                                    if decl.is_trait_method_dummy && !insertable {
                                        None
                                    } else {
                                        decl.subst(&type_mapping, engines);
                                        let new_ref = decl_engine
                                            .insert(decl)
                                            .with_parent(decl_engine, decl_ref.id().into());
                                        Some((
                                            name,
                                            ResolvedTraitImplItem::Typed(TyImplItem::Fn(new_ref)),
                                        ))
                                    }
                                }
                                ty::TyTraitItem::Constant(decl_ref) => {
                                    let mut decl = (*decl_engine.get(decl_ref.id())).clone();
                                    decl.subst(&type_mapping, engines);
                                    let new_ref = decl_engine.insert(decl);
                                    Some((
                                        name,
                                        ResolvedTraitImplItem::Typed(TyImplItem::Constant(new_ref)),
                                    ))
                                }
                                ty::TyTraitItem::Type(decl_ref) => {
                                    let mut decl = (*decl_engine.get(decl_ref.id())).clone();
                                    decl.subst(&type_mapping, engines);
                                    let new_ref = decl_engine.insert(decl);
                                    Some((
                                        name,
                                        ResolvedTraitImplItem::Typed(TyImplItem::Type(new_ref)),
                                    ))
                                }
                            },
                        })
                        .collect();
                    trait_map.insert_inner(
                        map_trait_name.clone(),
                        impl_span.clone(),
                        map_trait_decl_span.clone(),
                        *type_id,
                        trait_items,
                        engines,
                    );
                }
            }
        }
        trait_map
    }

    /// Find the entries in `self` that are equivalent to `type_id`.
    ///
    /// Notes:
    /// - equivalency is defined (1) based on whether the types contains types
    ///     that are dynamic and can change and (2) whether the types hold
    ///     equivalency after (1) is fulfilled
    /// - this method does not translate types from the found entries to the
    ///     `type_id` (like in `filter_by_type()`). This is because the only
    ///     entries that qualify as hits are equivalents of `type_id`
    pub(crate) fn get_items_for_type(
        &self,
        engines: &Engines,
        type_id: TypeId,
    ) -> Vec<ResolvedTraitImplItem> {
        self.get_items_and_trait_key_for_type(engines, type_id)
            .iter()
            .map(|i| i.0.clone())
            .collect::<Vec<_>>()
    }

    fn get_items_and_trait_key_for_type(
        &self,
        engines: &Engines,
        type_id: TypeId,
    ) -> Vec<(ResolvedTraitImplItem, TraitKey)> {
        let type_engine = engines.te();
        let unify_check = UnifyCheck::non_dynamic_equality(engines);

        let mut items = vec![];
        // small performance gain in bad case
        if matches!(&*type_engine.get(type_id), TypeInfo::ErrorRecovery(_)) {
            return items;
        }
        for entry in self.trait_impls.iter() {
            if unify_check.check(type_id, entry.key.type_id) {
                let mut trait_items = entry
                    .value
                    .trait_items
                    .values()
                    .cloned()
                    .map(|i| (i, entry.key.clone()))
                    .collect::<Vec<_>>();
                items.append(&mut trait_items);
            }
        }
        items
    }

    /// Find the spans of all impls for the given type.
    ///
    /// Notes:
    /// - equivalency is defined (1) based on whether the types contains types
    ///     that are dynamic and can change and (2) whether the types hold
    ///     equivalency after (1) is fulfilled
    /// - this method does not translate types from the found entries to the
    ///     `type_id` (like in `filter_by_type()`). This is because the only
    ///     entries that qualify as hits are equivalents of `type_id`
    pub(crate) fn get_impl_spans_for_type(&self, engines: &Engines, type_id: &TypeId) -> Vec<Span> {
        let type_engine = engines.te();
        let unify_check = UnifyCheck::non_dynamic_equality(engines);

        let mut spans = vec![];
        // small performance gain in bad case
        if matches!(&*type_engine.get(*type_id), TypeInfo::ErrorRecovery(_)) {
            return spans;
        }
        for entry in self.trait_impls.iter() {
            if unify_check.check(*type_id, entry.key.type_id) {
                spans.push(entry.value.impl_span.clone());
            }
        }
        spans
    }

    /// Find the entries in `self` with trait name `trait_name` and return the
    /// spans of the impls.
    pub(crate) fn get_impl_spans_for_trait_name(&self, trait_name: &CallPath) -> Vec<Span> {
        self.trait_impls
            .iter()
            .filter_map(|entry| {
                let map_trait_name = CallPath {
                    prefixes: entry.key.name.prefixes.clone(),
                    suffix: entry.key.name.suffix.name.clone(),
                    is_absolute: entry.key.name.is_absolute,
                };
                if &map_trait_name == trait_name {
                    return Some(entry.value.impl_span.clone());
                }
                None
            })
            .collect()
    }

    /// Find the entries in `self` that are equivalent to `type_id` with trait
    /// name `trait_name` and with trait type arguments.
    ///
    /// Notes:
    /// - equivalency is defined (1) based on whether the types contains types
    ///     that are dynamic and can change and (2) whether the types hold
    ///     equivalency after (1) is fulfilled
    /// - this method does not translate types from the found entries to the
    ///     `type_id` (like in `filter_by_type()`). This is because the only
    ///     entries that qualify as hits are equivalents of `type_id`
    pub(crate) fn get_items_for_type_and_trait_name_and_trait_type_arguments(
        &self,
        engines: &Engines,
        type_id: TypeId,
        trait_name: &CallPath,
        trait_type_args: &[TypeArgument],
    ) -> Vec<ResolvedTraitImplItem> {
        let type_engine = engines.te();
        let unify_check = UnifyCheck::non_dynamic_equality(engines);
        let mut items = vec![];
        // small performance gain in bad case
        if matches!(&*type_engine.get(type_id), TypeInfo::ErrorRecovery(_)) {
            return items;
        }
        for e in self.trait_impls.iter() {
            let map_trait_name = CallPath {
                prefixes: e.key.name.prefixes.clone(),
                suffix: e.key.name.suffix.name.clone(),
                is_absolute: e.key.name.is_absolute,
            };
            if &map_trait_name == trait_name
                && unify_check.check(type_id, e.key.type_id)
                && trait_type_args.len() == e.key.name.suffix.args.len()
                && trait_type_args
                    .iter()
                    .zip(e.key.name.suffix.args.iter())
                    .all(|(t1, t2)| unify_check.check(t1.type_id, t2.type_id))
            {
                let mut trait_items = e.value.trait_items.values().cloned().collect::<Vec<_>>();
                items.append(&mut trait_items);
            }
        }
        items
    }

    /// Find the entries in `self` that are equivalent to `type_id` with trait
    /// name `trait_name` and with trait type arguments.
    ///
    /// Notes:
    /// - equivalency is defined (1) based on whether the types contains types
    ///     that are dynamic and can change and (2) whether the types hold
    ///     equivalency after (1) is fulfilled
    /// - this method does not translate types from the found entries to the
    ///     `type_id` (like in `filter_by_type()`). This is because the only
    ///     entries that qualify as hits are equivalents of `type_id`
    pub(crate) fn get_items_for_type_and_trait_name_and_trait_type_arguments_typed(
        &self,
        engines: &Engines,
        type_id: TypeId,
        trait_name: &CallPath,
        trait_type_args: &[TypeArgument],
    ) -> Vec<ty::TyTraitItem> {
        self.get_items_for_type_and_trait_name_and_trait_type_arguments(
            engines,
            type_id,
            trait_name,
            trait_type_args,
        )
        .into_iter()
        .map(|item| item.expect_typed())
        .collect::<Vec<_>>()
    }

    pub(crate) fn get_trait_names_and_type_arguments_for_type(
        &self,
        engines: &Engines,
        type_id: TypeId,
    ) -> Vec<(CallPath, Vec<TypeArgument>)> {
        let type_engine = engines.te();
        let unify_check = UnifyCheck::non_dynamic_equality(engines);
        let mut trait_names = vec![];
        // small performance gain in bad case
        if matches!(&*type_engine.get(type_id), TypeInfo::ErrorRecovery(_)) {
            return trait_names;
        }
        for entry in self.trait_impls.iter() {
            if unify_check.check(type_id, entry.key.type_id) {
                let trait_call_path = CallPath {
                    prefixes: entry.key.name.prefixes.clone(),
                    suffix: entry.key.name.suffix.name.clone(),
                    is_absolute: entry.key.name.is_absolute,
                };
                trait_names.push((trait_call_path, entry.key.name.suffix.args.clone()));
            }
        }
        trait_names
    }

    pub(crate) fn get_trait_item_for_type(
        &self,
        handler: &Handler,
        engines: &Engines,
        symbol: &Ident,
        type_id: TypeId,
        as_trait: Option<CallPath>,
    ) -> Result<ResolvedTraitImplItem, ErrorEmitted> {
        let mut candidates = HashMap::<String, ResolvedTraitImplItem>::new();
        for (trait_item, trait_key) in self.get_items_and_trait_key_for_type(engines, type_id) {
            match trait_item {
                ResolvedTraitImplItem::Parsed(impl_item) => match impl_item {
                    ImplItem::Fn(fn_ref) => {
                        let decl = engines.pe().get_function(&fn_ref);
                        let trait_call_path_string = engines.help_out(&*trait_key.name).to_string();
                        if decl.name.as_str() == symbol.as_str()
                            && (as_trait.is_none()
                                || as_trait.clone().unwrap().to_string() == trait_call_path_string)
                        {
                            candidates.insert(
                                trait_call_path_string,
                                ResolvedTraitImplItem::Parsed(ImplItem::Fn(fn_ref)),
                            );
                        }
                    }
                    ImplItem::Constant(const_ref) => {
                        let decl = engines.pe().get_constant(&const_ref);
                        let trait_call_path_string = engines.help_out(&*trait_key.name).to_string();
                        if decl.name.as_str() == symbol.as_str()
                            && (as_trait.is_none()
                                || as_trait.clone().unwrap().to_string() == trait_call_path_string)
                        {
                            candidates.insert(
                                trait_call_path_string,
                                ResolvedTraitImplItem::Parsed(ImplItem::Constant(const_ref)),
                            );
                        }
                    }
                    ImplItem::Type(type_ref) => {
                        let decl = engines.pe().get_trait_type(&type_ref);
                        let trait_call_path_string = engines.help_out(&*trait_key.name).to_string();
                        if decl.name.as_str() == symbol.as_str()
                            && (as_trait.is_none()
                                || as_trait.clone().unwrap().to_string() == trait_call_path_string)
                        {
                            candidates.insert(
                                trait_call_path_string,
                                ResolvedTraitImplItem::Parsed(ImplItem::Type(type_ref)),
                            );
                        }
                    }
                },
                ResolvedTraitImplItem::Typed(ty_impl_item) => match ty_impl_item {
                    ty::TyTraitItem::Fn(fn_ref) => {
                        let decl = engines.de().get_function(&fn_ref);
                        let trait_call_path_string = engines.help_out(&*trait_key.name).to_string();
                        if decl.name.as_str() == symbol.as_str()
                            && (as_trait.is_none()
                                || as_trait.clone().unwrap().to_string() == trait_call_path_string)
                        {
                            candidates.insert(
                                trait_call_path_string,
                                ResolvedTraitImplItem::Typed(TyTraitItem::Fn(fn_ref)),
                            );
                        }
                    }
                    ty::TyTraitItem::Constant(const_ref) => {
                        let decl = engines.de().get_constant(&const_ref);
                        let trait_call_path_string = engines.help_out(&*trait_key.name).to_string();
                        if decl.call_path.suffix.as_str() == symbol.as_str()
                            && (as_trait.is_none()
                                || as_trait.clone().unwrap().to_string() == trait_call_path_string)
                        {
                            candidates.insert(
                                trait_call_path_string,
                                ResolvedTraitImplItem::Typed(TyTraitItem::Constant(const_ref)),
                            );
                        }
                    }
                    ty::TyTraitItem::Type(type_ref) => {
                        let decl = engines.de().get_type(&type_ref);
                        let trait_call_path_string = engines.help_out(&*trait_key.name).to_string();
                        if decl.name.as_str() == symbol.as_str()
                            && (as_trait.is_none()
                                || as_trait.clone().unwrap().to_string() == trait_call_path_string)
                        {
                            candidates.insert(
                                trait_call_path_string,
                                ResolvedTraitImplItem::Typed(TyTraitItem::Type(type_ref)),
                            );
                        }
                    }
                },
            }
        }

        match candidates.len().cmp(&1) {
            Ordering::Greater => Err(handler.emit_err(
                CompileError::MultipleApplicableItemsInScope {
                    item_name: symbol.as_str().to_string(),
                    item_kind: "item".to_string(),
                    type_name: engines.help_out(type_id).to_string(),
                    as_traits: candidates
                        .keys()
                        .map(|k| {
                            k.clone()
                                .split("::")
                                .collect::<Vec<_>>()
                                .last()
                                .unwrap()
                                .to_string()
                        })
                        .collect::<Vec<_>>(),
                    span: symbol.span(),
                },
            )),
            Ordering::Less => Err(handler.emit_err(CompileError::SymbolNotFound {
                name: symbol.clone(),
                span: symbol.span(),
            })),
            Ordering::Equal => Ok(candidates.values().next().unwrap().clone()),
        }
    }

    /// Checks to see if the trait constraints are satisfied for a given type.
    pub(crate) fn check_if_trait_constraints_are_satisfied_for_type(
        &mut self,
        handler: &Handler,
        type_id: TypeId,
        constraints: &[TraitConstraint],
        access_span: &Span,
        engines: &Engines,
        try_inserting_trait_impl_on_failure: TryInsertingTraitImplOnFailure,
    ) -> Result<(), ErrorEmitted> {
        let type_engine = engines.te();

        // resolving trait constraints require a concrete type, we need to default numeric to u64
        type_engine.decay_numeric(handler, engines, type_id, access_span)?;

        if constraints.is_empty() {
            return Ok(());
        }

        // Check we can use the cache
        let mut hasher = DefaultHasher::default();
        type_id.hash(&mut hasher);
        for c in constraints {
            c.hash(&mut hasher, engines);
        }
        let hash = hasher.finish();

        if self.satisfied_cache.contains(&hash) {
            return Ok(());
        }

        // Call the real implementation and cache when true
        match self.check_if_trait_constraints_are_satisfied_for_type_inner(
            handler,
            type_id,
            constraints,
            access_span,
            engines,
            try_inserting_trait_impl_on_failure,
        ) {
            Ok(()) => {
                self.satisfied_cache.insert(hash);
                Ok(())
            }
            r => r,
        }
    }

    fn check_if_trait_constraints_are_satisfied_for_type_inner(
        &mut self,
        handler: &Handler,
        type_id: TypeId,
        constraints: &[TraitConstraint],
        access_span: &Span,
        engines: &Engines,
        try_inserting_trait_impl_on_failure: TryInsertingTraitImplOnFailure,
    ) -> Result<(), ErrorEmitted> {
        let type_engine = engines.te();

        // If the type is generic/placeholder, its definition needs to contains all
        // constraints
        match &*type_engine.get(type_id) {
            TypeInfo::UnknownGeneric {
                trait_constraints, ..
            } => {
                let all = constraints.iter().all(|required| {
                    trait_constraints.iter().any(|constraint| {
                        constraint.eq(required, &PartialEqWithEnginesContext::new(engines))
                    })
                });
                if all {
                    return Ok(());
                }
            }
            TypeInfo::Placeholder(p) => {
                let all = constraints.iter().all(|required| {
                    p.trait_constraints.iter().any(|constraint| {
                        constraint.eq(required, &PartialEqWithEnginesContext::new(engines))
                    })
                });
                if all {
                    return Ok(());
                }
            }
            _ => {}
        }

        let _decl_engine = engines.de();
        let unify_check = UnifyCheck::non_dynamic_equality(engines);

        let all_impld_traits: BTreeSet<(Ident, TypeId)> = self
            .trait_impls
            .iter()
            .filter_map(|e| {
                let key = &e.key;
                let suffix = &key.name.suffix;
                if unify_check.check(type_id, key.type_id) {
                    let map_trait_type_id = type_engine.insert(
                        engines,
                        TypeInfo::Custom {
                            qualified_call_path: suffix.name.clone().into(),
                            type_arguments: if suffix.args.is_empty() {
                                None
                            } else {
                                Some(suffix.args.to_vec())
                            },
                            root_type_id: None,
                        },
                        suffix.name.span().source_id(),
                    );
                    Some((suffix.name.clone(), map_trait_type_id))
                } else {
                    None
                }
            })
            .collect();

        let required_traits: BTreeSet<(Ident, TypeId)> = constraints
            .iter()
            .map(|c| {
                let TraitConstraint {
                    trait_name: constraint_trait_name,
                    type_arguments: constraint_type_arguments,
                } = c;
                let constraint_type_id = type_engine.insert(
                    engines,
                    TypeInfo::Custom {
                        qualified_call_path: constraint_trait_name.suffix.clone().into(),
                        type_arguments: if constraint_type_arguments.is_empty() {
                            None
                        } else {
                            Some(constraint_type_arguments.clone())
                        },
                        root_type_id: None,
                    },
                    constraint_trait_name.span().source_id(),
                );
                (c.trait_name.suffix.clone(), constraint_type_id)
            })
            .collect();

        let traits_not_found: BTreeSet<(BaseIdent, TypeId)> = required_traits
            .into_iter()
            .filter(|(required_trait_name, required_trait_type_id)| {
                !all_impld_traits
                    .iter()
                    .any(|(trait_name, constraint_type_id)| {
                        trait_name == required_trait_name
                            && unify_check.check(*constraint_type_id, *required_trait_type_id)
                    })
            })
            .collect();

        handler.scope(|handler| {
            for (trait_name, constraint_type_id) in traits_not_found.iter() {
                if matches!(
                    try_inserting_trait_impl_on_failure,
                    TryInsertingTraitImplOnFailure::Yes
                ) {
                    self.insert_for_type(engines, type_id);
                    return self.check_if_trait_constraints_are_satisfied_for_type(
                        handler,
                        type_id,
                        constraints,
                        access_span,
                        engines,
                        TryInsertingTraitImplOnFailure::No,
                    );
                } else {
                    let mut type_arguments_string = "".to_string();
                    if let TypeInfo::Custom {
                        qualified_call_path: _,
                        type_arguments: Some(type_arguments),
                        root_type_id: _,
                    } = &*type_engine.get(*constraint_type_id)
                    {
                        type_arguments_string = format!("<{}>", engines.help_out(type_arguments));
                    }

                    // TODO: use a better span
                    handler.emit_err(CompileError::TraitConstraintNotSatisfied {
                        type_id: type_id.index(),
                        ty: engines.help_out(type_id).to_string(),
                        trait_name: format!("{}{}", trait_name, type_arguments_string),
                        span: access_span.clone(),
                    });
                }
            }

            Ok(())
        })
    }
}
