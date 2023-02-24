use std::collections::{BTreeMap, BTreeSet};

use sway_error::error::CompileError;
use sway_types::{Ident, Span, Spanned};

use crate::{
    decl_engine::DeclRef,
    engine_threading::*,
    error::*,
    language::{
        ty::{self, TyImplItem},
        CallPath,
    },
    type_system::*,
};

use super::*;

/// Map holding trait implementations for types.
///
/// Note: "impl self" blocks are considered traits and are stored in the
/// [TraitMap].
#[derive(Clone, Debug, Default)]
pub(crate) struct TraitMap {
    trait_impls: TraitImpls,
}

impl TraitMap {
    /// Given a [TraitCallPath] `trait_name`, [TypeId] `type_id`, and list of
    /// [TyImplItem](ty::TyImplItem) `items`, inserts
    /// `items` into the [TraitMap] with the key `(trait_name, type_id)`.
    ///
    /// This method is as conscious as possible of existing entries in the
    /// [TraitMap], and tries to append `items` to an existing list of
    /// declarations for the key `(trait_name, type_id)` whenever possible.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn insert(
        &mut self,
        trait_call_path: CallPath,
        trait_type_args: Vec<TypeArgument>,
        type_id: TypeId,
        items: &[TyImplItem],
        impl_span: &Span,
        is_impl_self: bool,
        engines: Engines<'_>,
    ) -> CompileResult<()> {
        let warnings = vec![];
        let mut errors = vec![];

        let type_engine = engines.te();
        let decl_engine = engines.de();

        let mut trait_items: TraitItems = im::OrdMap::new();
        for item in items.iter() {
            match item {
                TyImplItem::Fn(decl_ref) => {
                    trait_items.insert(decl_ref.name.to_string(), item.clone());
                }
            }
        }

        // check to see if adding this trait will produce a conflicting definition
        let trait_type_id = type_engine.insert(
            decl_engine,
            TypeInfo::Custom {
                call_path: trait_call_path.suffix.clone().into(),
                type_arguments: if trait_type_args.is_empty() {
                    None
                } else {
                    Some(trait_type_args.clone())
                },
            },
        );
        for (key, map_trait_items) in self.trait_impls.iter() {
            let TraitKey {
                call_path: map_call_path,
                implementing_for: map_implementing_for,
            } = key;
            let CallPath {
                suffix:
                    TraitSuffix {
                        name: map_name,
                        args: map_trait_type_args,
                    },
                ..
            } = map_call_path;
            let map_trait_type_id = type_engine.insert(
                decl_engine,
                TypeInfo::Custom {
                    call_path: map_name.clone().into(),
                    type_arguments: if map_trait_type_args.is_empty() {
                        None
                    } else {
                        Some(map_trait_type_args.to_vec())
                    },
                },
            );

            let unify_checker = UnifyCheck::new(engines).set_strict();
            let types_are_subset = unify_checker.check(type_id, *map_implementing_for);
            let traits_are_subset = unify_checker.check(trait_type_id, map_trait_type_id);

            if types_are_subset && traits_are_subset && !is_impl_self {
                let trait_name_str = format!(
                    "{}{}",
                    trait_call_path.suffix,
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
                errors.push(CompileError::ConflictingImplsForTraitAndType {
                    trait_name: trait_name_str,
                    type_implementing_for: engines.help_out(type_id).to_string(),
                    second_impl_span: impl_span.clone(),
                });
            } else if types_are_subset && (traits_are_subset || is_impl_self) {
                for (name, item) in trait_items.iter() {
                    match item {
                        ty::TyTraitItem::Fn(decl_ref) => {
                            if map_trait_items.get(name).is_some() {
                                errors.push(CompileError::DuplicateDeclDefinedForType {
                                    decl_kind: "method".into(),
                                    decl_name: decl_ref.name.to_string(),
                                    type_implementing_for: engines.help_out(type_id).to_string(),
                                    span: decl_ref.name.span(),
                                });
                            }
                        }
                    }
                }
            }
        }
        let trait_name: TraitCallPath = CallPath {
            prefixes: trait_call_path.prefixes,
            suffix: TraitSuffix {
                name: trait_call_path.suffix,
                args: trait_type_args,
            },
            is_absolute: trait_call_path.is_absolute,
        };

        // even if there is a conflicting definition, add the trait anyway
        self.insert_inner(trait_name, type_id, trait_items, engines);

        if errors.is_empty() {
            ok((), warnings, errors)
        } else {
            err(warnings, errors)
        }
    }

    fn insert_inner(
        &mut self,
        call_path: TraitCallPath,
        implementing_for: TypeId,
        trait_methods: TraitItems,
        engines: Engines<'_>,
    ) {
        let key = TraitKey {
            call_path,
            implementing_for,
        };
        let mut trait_impls = im::OrdMap::new();
        trait_impls.insert(key, trait_methods);
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
    pub(crate) fn insert_for_type(&mut self, engines: Engines<'_>, type_id: TypeId) {
        self.extend(self.filter_by_type(type_id, engines), engines);
    }

    /// Given [TraitMap]s `self` and `other`, extend `self` with `other`,
    /// extending existing entries when possible.
    pub(crate) fn extend(&mut self, other: TraitMap, _engines: Engines<'_>) {
        for (key, value) in other.trait_impls.into_iter() {
            match self.trait_impls.entry(key) {
                im::ordmap::Entry::Occupied(mut o) => {
                    o.get_mut().extend(value);
                }
                im::ordmap::Entry::Vacant(v) => {
                    v.insert(value);
                }
            }
        }
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
    /// library data;
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
    /// dep data;
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
    pub(crate) fn filter_by_type(&self, type_id: TypeId, engines: Engines<'_>) -> TraitMap {
        let type_engine = engines.te();
        let unify_checker = UnifyCheck::new(engines).set_strict();
        // a curried version of the decider protocol to use in the helper functions
        let decider = |left: TypeId, right: TypeId| unify_checker.check(left, right);
        let mut all_types = type_engine.get(type_id).extract_inner_types(type_engine);
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
    /// library my_double;
    ///
    /// pub trait MyDouble<T> {
    ///     fn my_double(self, input: T) -> T;
    /// }
    /// ```
    ///
    /// `my_point.sw`:
    /// ```ignore
    /// library my_point;
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
    /// dep my_double;
    /// dep my_point;
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
        engines: Engines<'_>,
    ) -> TraitMap {
        let type_engine = engines.te();
        let unify_checker = UnifyCheck::new(engines).set_strict();
        // a curried version of the decider protocol to use in the helper functions
        let decider = |left: TypeId, right: TypeId| {
            unify_checker.check(left, right) || unify_checker.check(right, left)
        };
        let mut trait_map = self.filter_by_type_inner(engines, vec![type_id], decider);
        let all_types = type_engine
            .get(type_id)
            .extract_inner_types(type_engine)
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
        engines: Engines<'_>,
        mut all_types: Vec<TypeId>,
        decider: impl Fn(TypeId, TypeId) -> bool,
    ) -> TraitMap {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        let mut trait_map = TraitMap::default();
        for (key, map_trait_items) in self.trait_impls.iter() {
            let TraitKey {
                call_path: map_call_path,
                implementing_for: map_implementing_for,
            } = key;
            for type_id in all_types.iter_mut() {
                let type_info = type_engine.get(*type_id);
                if !type_info.can_change() && *type_id == *map_implementing_for {
                    trait_map.insert_inner(
                        map_call_path.clone(),
                        *type_id,
                        map_trait_items.clone(),
                        engines,
                    );
                } else if decider(*type_id, *map_implementing_for) {
                    let type_mapping = TypeSubstMap::from_superset_and_subset(
                        type_engine,
                        *map_implementing_for,
                        *type_id,
                    );
                    let new_self_type = type_engine.insert(decl_engine, TypeInfo::SelfType);
                    type_id.replace_self_type(engines, new_self_type);
                    let trait_items: TraitItems = map_trait_items
                        .clone()
                        .into_iter()
                        .map(|(name, item)| {
                            #[allow(clippy::infallible_destructuring_match)]
                            let decl_ref = match &item {
                                ty::TyTraitItem::Fn(decl_ref) => decl_ref,
                            };
                            let mut decl = decl_engine.get(decl_ref);
                            decl.subst(&type_mapping, engines);
                            decl.replace_self_type(engines, new_self_type);
                            let new_ref = decl_engine
                                .insert_wrapper(decl_ref.name.clone(), decl, decl_ref.span())
                                .with_parent(decl_engine, decl_ref);
                            let item = match item {
                                ty::TyTraitItem::Fn(_) => TyImplItem::Fn(new_ref),
                            };
                            (name, item)
                        })
                        .collect();
                    trait_map.insert_inner(map_call_path.clone(), *type_id, trait_items, engines);
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
    pub(crate) fn get_methods_for_type(
        &self,
        engines: Engines<'_>,
        type_id: TypeId,
    ) -> Vec<DeclRef> {
        let type_engine = engines.te();
        let mut methods: Vec<DeclRef> = vec![];
        // small performance gain in bad case
        if type_engine
            .get(type_id)
            .eq(&TypeInfo::ErrorRecovery, engines)
        {
            return vec![];
        }
        for (key, value) in self.trait_impls.iter() {
            let trait_items = value
                .values()
                .cloned()
                .into_iter()
                .flat_map(|item| match item {
                    ty::TyTraitItem::Fn(decl_ref) => Some(decl_ref),
                });
            if type_id == key.implementing_for
                || type_engine
                    .get(type_id)
                    .eq(&type_engine.get(key.implementing_for), engines)
            {
                methods.extend(trait_items.collect::<Vec<_>>());
            }
        }
        methods
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
    pub(crate) fn get_methods_for_type_and_trait_name(
        &self,
        engines: Engines<'_>,
        type_id: TypeId,
        trait_name: &CallPath,
    ) -> Vec<DeclRef> {
        let type_engine = engines.te();
        let mut methods = vec![];
        // small performance gain in bad case
        if type_engine
            .get(type_id)
            .eq(&TypeInfo::ErrorRecovery, engines)
        {
            return methods;
        }
        for (key, value) in self.trait_impls.iter() {
            let map_trait_name = CallPath {
                prefixes: key.call_path.prefixes.clone(),
                suffix: key.call_path.suffix.name.clone(),
                is_absolute: key.call_path.is_absolute,
            };
            if &map_trait_name == trait_name
                || type_engine
                    .get(type_id)
                    .eq(&type_engine.get(key.implementing_for), engines)
            {
                let mut trait_methods = value
                    .values()
                    .cloned()
                    .into_iter()
                    .flat_map(|item| match item {
                        ty::TyTraitItem::Fn(decl_ref) => Some(decl_ref),
                    })
                    .collect::<Vec<_>>();
                methods.append(&mut trait_methods);
            }
        }
        methods
    }

    /// Checks to see if the trait constraints are satisfied for a given type.
    pub(crate) fn check_if_trait_constraints_are_satisfied_for_type(
        &self,
        type_id: TypeId,
        constraints: &[TraitConstraint],
        access_span: &Span,
        engines: Engines<'_>,
    ) -> CompileResult<()> {
        let warnings = vec![];
        let mut errors = vec![];

        let type_engine = engines.te();
        let decl_engine = engines.de();

        let all_impld_traits: BTreeMap<Ident, TypeId> = self
            .trait_impls
            .iter()
            .filter_map(|(key, _)| {
                let suffix = &key.call_path.suffix;
                let map_trait_type_id = type_engine.insert(
                    decl_engine,
                    TypeInfo::Custom {
                        call_path: suffix.name.clone().into(),
                        type_arguments: if suffix.args.is_empty() {
                            None
                        } else {
                            Some(suffix.args.to_vec())
                        },
                    },
                );
                if type_engine
                    .get(type_id)
                    .eq(&type_engine.get(key.implementing_for), engines)
                {
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
                    decl_engine,
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
                    Some(constraint_type_id) => type_engine
                        .get(*constraint_type_id)
                        .eq(&type_engine.get(*impld_trait_type_id), engines),
                    _ => false,
                }
            })
            .collect();

        let required_traits_names: BTreeSet<Ident> = required_traits.keys().cloned().collect();
        let relevant_impld_traits_names: BTreeSet<Ident> =
            relevant_impld_traits.keys().cloned().collect();

        for trait_name in required_traits_names.difference(&relevant_impld_traits_names) {
            // TODO: use a better span
            errors.push(CompileError::TraitConstraintNotSatisfied {
                ty: engines.help_out(type_id).to_string(),
                trait_name: trait_name.to_string(),
                span: access_span.clone(),
            });
        }

        if errors.is_empty() {
            ok((), warnings, errors)
        } else {
            err(warnings, errors)
        }
    }
}

impl DisplayWithEngines for TraitMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, engines: Engines<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "\n+++++\n{{\n{}\n}}\n+++++",
            self.trait_impls
                .iter()
                .map(|(key, items)| {
                    format!(
                        "   {}:\n       -> {}",
                        engines.help_out(key),
                        items
                            .iter()
                            .map(|(name, _)| { name.to_string() })
                            .collect::<Vec<_>>()
                            .join("\n       -> ")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}
