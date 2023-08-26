use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Span, Spanned};

use crate::{
    decl_engine::{DeclEngineGet, DeclEngineInsert},
    engine_threading::*,
    language::{
        ty::{self, TyImplItem},
        CallPath,
    },
    type_system::{SubstTypes, TypeId},
    TraitConstraint, TypeArgument, TypeInfo, TypeSubstMap, UnifyCheck,
};

use super::TryInsertingTraitImplOnFailure;

#[derive(Clone, Debug)]
struct TraitSuffix {
    name: Ident,
    args: Vec<TypeArgument>,
}
impl PartialEqWithEngines for TraitSuffix {
    fn eq(&self, other: &Self, engines: &Engines) -> bool {
        self.name == other.name && self.args.eq(&other.args, engines)
    }
}
impl OrdWithEngines for TraitSuffix {
    fn cmp(&self, other: &Self, engines: &Engines) -> std::cmp::Ordering {
        self.name
            .cmp(&other.name)
            .then_with(|| self.args.cmp(&other.args, engines))
    }
}

impl<T: PartialEqWithEngines> PartialEqWithEngines for CallPath<T> {
    fn eq(&self, other: &Self, engines: &Engines) -> bool {
        self.prefixes == other.prefixes
            && self.suffix.eq(&other.suffix, engines)
            && self.is_absolute == other.is_absolute
    }
}
impl<T: OrdWithEngines> OrdWithEngines for CallPath<T> {
    fn cmp(&self, other: &Self, engines: &Engines) -> Ordering {
        self.prefixes
            .cmp(&other.prefixes)
            .then_with(|| self.suffix.cmp(&other.suffix, engines))
            .then_with(|| self.is_absolute.cmp(&other.is_absolute))
    }
}

type TraitName = CallPath<TraitSuffix>;

#[derive(Clone, Debug)]
struct TraitKey {
    name: TraitName,
    type_id: TypeId,
    trait_decl_span: Option<Span>,
}

impl OrdWithEngines for TraitKey {
    fn cmp(&self, other: &Self, engines: &Engines) -> std::cmp::Ordering {
        self.name
            .cmp(&other.name, engines)
            .then_with(|| self.type_id.cmp(&other.type_id))
    }
}

/// Map of name to [TyImplItem](ty::TyImplItem)
type TraitItems = im::HashMap<String, TyImplItem>;

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
        items: &[TyImplItem],
        impl_span: &Span,
        trait_decl_span: Option<Span>,
        is_impl_self: bool,
        engines: &Engines,
    ) -> Result<(), ErrorEmitted> {
        let type_engine = engines.te();
        let _decl_engine = engines.de();

        handler.scope(|handler| {
            let mut trait_items: TraitItems = im::HashMap::new();
            for item in items.iter() {
                match item {
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
                }
            }

            // check to see if adding this trait will produce a conflicting definition
            let trait_type_id = type_engine.insert(
                engines,
                TypeInfo::Custom {
                    call_path: trait_name.suffix.clone().into(),
                    type_arguments: if trait_type_args.is_empty() {
                        None
                    } else {
                        Some(trait_type_args.clone())
                    },
                },
            );
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
                        ..
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
                } = map_trait_name;
                let map_trait_type_id = type_engine.insert(
                    engines,
                    TypeInfo::Custom {
                        call_path: map_trait_name_suffix.clone().into(),
                        type_arguments: if map_trait_type_args.is_empty() {
                            None
                        } else {
                            Some(map_trait_type_args.to_vec())
                        },
                    },
                );

                let unify_checker = UnifyCheck::constraint_subset(engines);
                let types_are_subset = unify_checker.check(type_id, *map_type_id);
                let traits_are_subset = unify_checker.check(trait_type_id, map_trait_type_id);

                if types_are_subset && traits_are_subset && !is_impl_self {
                    let trait_name_str = format!(
                        "{}{}",
                        trait_name.suffix,
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
                        second_impl_span: impl_span.clone(),
                    });
                } else if types_are_subset && (traits_are_subset || is_impl_self) {
                    for (name, item) in trait_items.iter() {
                        match item {
                            ty::TyTraitItem::Fn(decl_ref) => {
                                if map_trait_items.get(name).is_some() {
                                    handler.emit_err(CompileError::DuplicateDeclDefinedForType {
                                        decl_kind: "method".into(),
                                        decl_name: decl_ref.name().to_string(),
                                        type_implementing_for: engines
                                            .help_out(type_id)
                                            .to_string(),
                                        span: decl_ref.name().span(),
                                    });
                                }
                            }
                            ty::TyTraitItem::Constant(decl_ref) => {
                                if map_trait_items.get(name).is_some() {
                                    handler.emit_err(CompileError::DuplicateDeclDefinedForType {
                                        decl_kind: "constant".into(),
                                        decl_name: decl_ref.name().to_string(),
                                        type_implementing_for: engines
                                            .help_out(type_id)
                                            .to_string(),
                                        span: decl_ref.name().span(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            let trait_name: TraitName = CallPath {
                prefixes: trait_name.prefixes,
                suffix: TraitSuffix {
                    name: trait_name.suffix,
                    args: trait_type_args,
                },
                is_absolute: trait_name.is_absolute,
            };

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
        let trait_map = TraitMap { trait_impls };

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
                .binary_search_by(|se| se.key.cmp(&oe.key, engines));

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
        let type_engine = engines.te();

        let unify_checker = UnifyCheck::constraint_subset(engines);

        // a curried version of the decider protocol to use in the helper functions
        let decider = |left: TypeId, right: TypeId| unify_checker.check(left, right);
        let mut all_types = type_engine.get(type_id).extract_inner_types(engines);
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
        let type_engine = engines.te();

        let unify_checker = UnifyCheck::constraint_subset(engines);
        let unify_checker_for_item_import = UnifyCheck::non_generic_constraint_subset(engines);

        // a curried version of the decider protocol to use in the helper functions
        let decider = |left: TypeId, right: TypeId| {
            unify_checker.check(left, right) || unify_checker_for_item_import.check(right, left)
        };
        let mut trait_map = self.filter_by_type_inner(engines, vec![type_id], decider);
        let all_types = type_engine
            .get(type_id)
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
                        .map(|(name, item)| match &item {
                            ty::TyTraitItem::Fn(decl_ref) => {
                                let mut decl = decl_engine.get(decl_ref.id());
                                decl.subst(&type_mapping, engines);
                                let new_ref = decl_engine
                                    .insert(decl)
                                    .with_parent(decl_engine, decl_ref.id().into());
                                (name, TyImplItem::Fn(new_ref))
                            }
                            ty::TyTraitItem::Constant(decl_ref) => {
                                let mut decl = decl_engine.get(decl_ref.id());
                                decl.subst(&type_mapping, engines);
                                let new_ref = decl_engine.insert(decl);
                                (name, TyImplItem::Constant(new_ref))
                            }
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
    ) -> Vec<ty::TyTraitItem> {
        let type_engine = engines.te();
        let unify_check = UnifyCheck::non_dynamic_equality(engines);

        let mut items = vec![];
        // small performance gain in bad case
        if matches!(type_engine.get(type_id), TypeInfo::ErrorRecovery(_)) {
            return items;
        }
        for entry in self.trait_impls.iter() {
            if unify_check.check(type_id, entry.key.type_id) {
                let mut trait_items = entry
                    .value
                    .trait_items
                    .values()
                    .cloned()
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
        if matches!(type_engine.get(*type_id), TypeInfo::ErrorRecovery(_)) {
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
    /// name `trait_name`.
    ///
    /// Notes:
    /// - equivalency is defined (1) based on whether the types contains types
    ///     that are dynamic and can change and (2) whether the types hold
    ///     equivalency after (1) is fulfilled
    /// - this method does not translate types from the found entries to the
    ///     `type_id` (like in `filter_by_type()`). This is because the only
    ///     entries that qualify as hits are equivalents of `type_id`
    pub(crate) fn get_items_for_type_and_trait_name(
        &self,
        engines: &Engines,
        type_id: TypeId,
        trait_name: &CallPath,
    ) -> Vec<ty::TyTraitItem> {
        let type_engine = engines.te();
        let unify_check = UnifyCheck::non_dynamic_equality(engines);
        let mut items = vec![];
        // small performance gain in bad case
        if matches!(type_engine.get(type_id), TypeInfo::ErrorRecovery(_)) {
            return items;
        }
        for e in self.trait_impls.iter() {
            let map_trait_name = CallPath {
                prefixes: e.key.name.prefixes.clone(),
                suffix: e.key.name.suffix.name.clone(),
                is_absolute: e.key.name.is_absolute,
            };
            if &map_trait_name == trait_name && unify_check.check(type_id, e.key.type_id) {
                let mut trait_items = e.value.trait_items.values().cloned().collect::<Vec<_>>();
                items.append(&mut trait_items);
            }
        }
        items
    }

    pub(crate) fn get_trait_names_for_type(
        &self,
        engines: &Engines,
        type_id: TypeId,
    ) -> Vec<CallPath> {
        let type_engine = engines.te();
        let unify_check = UnifyCheck::non_dynamic_equality(engines);
        let mut trait_names = vec![];
        // small performance gain in bad case
        if matches!(type_engine.get(type_id), TypeInfo::ErrorRecovery(_)) {
            return trait_names;
        }
        for entry in self.trait_impls.iter() {
            if unify_check.check(type_id, entry.key.type_id) {
                let trait_call_path = CallPath {
                    prefixes: entry.key.name.prefixes.clone(),
                    suffix: entry.key.name.suffix.name.clone(),
                    is_absolute: entry.key.name.is_absolute,
                };
                trait_names.push(trait_call_path);
            }
        }
        trait_names
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
        let _decl_engine = engines.de();
        let unify_check = UnifyCheck::non_dynamic_equality(engines);

        // resolving trait constraits require a concrete type, we need to default numeric to u64
        type_engine.decay_numeric(handler, engines, type_id, access_span)?;

        let all_impld_traits: BTreeMap<Ident, TypeId> = self
            .trait_impls
            .iter()
            .filter_map(|e| {
                let key = &e.key;
                let suffix = &key.name.suffix;
                let map_trait_type_id = type_engine.insert(
                    engines,
                    TypeInfo::Custom {
                        call_path: suffix.name.clone().into(),
                        type_arguments: if suffix.args.is_empty() {
                            None
                        } else {
                            Some(suffix.args.to_vec())
                        },
                    },
                );
                if unify_check.check(type_id, key.type_id) {
                    Some((suffix.name.clone(), map_trait_type_id))
                } else {
                    None
                }
            })
            .collect();

        let required_traits: BTreeMap<Ident, TypeId> = constraints
            .iter()
            .map(|c| {
                let TraitConstraint {
                    trait_name: constraint_trait_name,
                    type_arguments: constraint_type_arguments,
                } = c;
                let constraint_type_id = type_engine.insert(
                    engines,
                    TypeInfo::Custom {
                        call_path: constraint_trait_name.suffix.clone().into(),
                        type_arguments: if constraint_type_arguments.is_empty() {
                            None
                        } else {
                            Some(constraint_type_arguments.clone())
                        },
                    },
                );
                (c.trait_name.suffix.clone(), constraint_type_id)
            })
            .collect();

        let relevant_impld_traits: BTreeMap<Ident, TypeId> = all_impld_traits
            .into_iter()
            .filter(|(impld_trait_name, impld_trait_type_id)| {
                match required_traits.get(impld_trait_name) {
                    Some(constraint_type_id) => {
                        unify_check.check(*constraint_type_id, *impld_trait_type_id)
                    }
                    _ => false,
                }
            })
            .collect();

        let required_traits_names: BTreeSet<Ident> = required_traits.keys().cloned().collect();
        let relevant_impld_traits_names: BTreeSet<Ident> =
            relevant_impld_traits.keys().cloned().collect();

        handler.scope(|handler| {
            for trait_name in required_traits_names.difference(&relevant_impld_traits_names) {
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
                    // TODO: use a better span
                    handler.emit_err(CompileError::TraitConstraintNotSatisfied {
                        ty: engines.help_out(type_id).to_string(),
                        trait_name: trait_name.to_string(),
                        span: access_span.clone(),
                    });
                }
            }
            Ok(())
        })
    }
}
