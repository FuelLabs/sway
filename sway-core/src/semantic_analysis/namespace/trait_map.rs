use std::collections::BTreeSet;

use sway_error::error::CompileError;
use sway_types::{Ident, Span, Spanned};

use crate::{
    declaration_engine::{de_get_function, de_insert, de_look_up_decl_id, DeclarationId},
    error::*,
    language::CallPath,
    type_system::{CopyTypes, TypeId},
    OrdWithTypeEngine, PartialEqWithTypeEngine, ReplaceSelfType, TraitConstraint, TypeArgument,
    TypeEngine, TypeInfo, TypeMapping,
};

#[derive(Clone, Debug)]
struct TraitSuffix {
    name: Ident,
    args: Vec<TypeArgument>,
}
impl PartialEqWithTypeEngine for TraitSuffix {
    fn eq(&self, rhs: &Self, type_engine: &TypeEngine) -> bool {
        self.name == rhs.name && self.args.eq(&rhs.args, type_engine)
    }
}
impl OrdWithTypeEngine for TraitSuffix {
    fn cmp(&self, rhs: &Self, type_engine: &TypeEngine) -> std::cmp::Ordering {
        self.name
            .cmp(&rhs.name)
            .then_with(|| self.args.cmp(&rhs.args, type_engine))
    }
}

impl<T: PartialEqWithTypeEngine> PartialEqWithTypeEngine for CallPath<T> {
    fn eq(&self, rhs: &Self, type_engine: &TypeEngine) -> bool {
        self.prefixes == rhs.prefixes
            && self.suffix.eq(&rhs.suffix, type_engine)
            && self.is_absolute == rhs.is_absolute
    }
}
impl<T: OrdWithTypeEngine> OrdWithTypeEngine for CallPath<T> {
    fn cmp(&self, rhs: &Self, type_engine: &TypeEngine) -> std::cmp::Ordering {
        self.prefixes
            .cmp(&rhs.prefixes)
            .then_with(|| self.suffix.cmp(&rhs.suffix, type_engine))
            .then_with(|| self.is_absolute.cmp(&rhs.is_absolute))
    }
}

type TraitName = CallPath<TraitSuffix>;

#[derive(Clone, Debug)]
struct TraitKey {
    name: TraitName,
    type_id: TypeId,
}

impl OrdWithTypeEngine for TraitKey {
    fn cmp(&self, rhs: &Self, type_engine: &TypeEngine) -> std::cmp::Ordering {
        self.name
            .cmp(&rhs.name, type_engine)
            .then_with(|| self.type_id.cmp(&rhs.type_id))
    }
}

/// Map of function name to [TyFunctionDeclaration](ty::TyFunctionDeclaration)
type TraitMethods = im::HashMap<String, DeclarationId>;

#[derive(Clone, Debug)]
struct TraitEntry {
    key: TraitKey,
    value: TraitMethods,
}

/// Map of trait name and type to [TraitMethods].
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
    /// [TyFunctionDeclaration](ty::TyFunctionDeclaration) `methods`, inserts
    /// `methods` into the [TraitMap] with the key `(trait_name, type_id)`.
    ///
    /// This method is as conscious as possible of existing entries in the
    /// [TraitMap], and tries to append `methods` to an existing list of
    /// [TyFunctionDeclaration](ty::TyFunctionDeclaration) for the key
    /// `(trait_name, type_id)` whenever possible.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn insert(
        &mut self,
        trait_name: CallPath,
        trait_type_args: Vec<TypeArgument>,
        type_id: TypeId,
        methods: &[DeclarationId],
        impl_span: &Span,
        is_impl_self: bool,
        type_engine: &TypeEngine,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let mut trait_methods: TraitMethods = im::HashMap::new();
        for decl_id in methods.iter() {
            let method = check!(
                CompileResult::from(de_get_function(decl_id.clone(), impl_span)),
                return err(warnings, errors),
                warnings,
                errors
            );
            trait_methods.insert(method.name.to_string(), decl_id.clone());
        }

        // check to see if adding this trait will produce a conflicting definition
        let trait_type_id = type_engine.insert_type(TypeInfo::Custom {
            name: trait_name.suffix.clone(),
            type_arguments: if trait_type_args.is_empty() {
                None
            } else {
                Some(trait_type_args.clone())
            },
        });
        for TraitEntry {
            key:
                TraitKey {
                    name: map_trait_name,
                    type_id: map_type_id,
                },
            value: map_trait_methods,
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
            let map_trait_type_id = type_engine.insert_type(TypeInfo::Custom {
                name: map_trait_name_suffix.clone(),
                type_arguments: if map_trait_type_args.is_empty() {
                    None
                } else {
                    Some(map_trait_type_args.to_vec())
                },
            });

            let types_are_subset = type_engine
                .look_up_type_id(type_id)
                .is_subset_of(&type_engine.look_up_type_id(*map_type_id), type_engine);
            let traits_are_subset = type_engine
                .look_up_type_id(trait_type_id)
                .is_subset_of(&type_engine.look_up_type_id(map_trait_type_id), type_engine);

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
                                .map(|type_arg| type_engine.help_out(type_arg).to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    }
                );
                errors.push(CompileError::ConflictingImplsForTraitAndType {
                    trait_name: trait_name_str,
                    type_implementing_for: type_engine.help_out(type_id).to_string(),
                    second_impl_span: impl_span.clone(),
                });
            } else if types_are_subset {
                for (name, decl_id) in trait_methods.iter() {
                    if map_trait_methods.get(name).is_some() {
                        let method = check!(
                            CompileResult::from(de_get_function(decl_id.clone(), impl_span)),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        errors.push(CompileError::DuplicateMethodsDefinedForType {
                            func_name: method.name.to_string(),
                            type_implementing_for: type_engine.help_out(type_id).to_string(),
                            span: method.name.span(),
                        });
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
        self.insert_inner(trait_name, type_id, trait_methods, type_engine);

        if errors.is_empty() {
            ok((), warnings, errors)
        } else {
            err(warnings, errors)
        }
    }

    fn insert_inner(
        &mut self,
        trait_name: TraitName,
        type_id: TypeId,
        trait_methods: TraitMethods,
        type_engine: &TypeEngine,
    ) {
        let key = TraitKey {
            name: trait_name,
            type_id,
        };
        let entry = TraitEntry {
            key,
            value: trait_methods,
        };
        let trait_impls: TraitImpls = vec![entry];
        let trait_map = TraitMap { trait_impls };

        self.extend(trait_map, type_engine);
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
    pub(crate) fn insert_for_type(&mut self, type_engine: &TypeEngine, type_id: TypeId) {
        self.extend(self.filter_by_type(type_id, type_engine), type_engine);
    }

    /// Given [TraitMap]s `self` and `other`, extend `self` with `other`,
    /// extending existing entries when possible.
    pub(crate) fn extend(&mut self, other: TraitMap, type_engine: &TypeEngine) {
        for oe in other.trait_impls.into_iter() {
            let pos = self
                .trait_impls
                .binary_search_by(|se| se.key.cmp(&oe.key, type_engine));

            match pos {
                Ok(pos) => self.trait_impls[pos].value.extend(oe.value.into_iter()),
                Err(pos) => self.trait_impls.insert(pos, oe),
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
    pub(crate) fn filter_by_type(&self, type_id: TypeId, type_engine: &TypeEngine) -> TraitMap {
        // a curried version of the decider protocol to use in the helper functions
        let decider = |type_info: &TypeInfo, map_type_info: &TypeInfo| {
            type_info.is_subset_of(map_type_info, type_engine)
        };
        let mut all_types = type_engine
            .look_up_type_id(type_id)
            .extract_inner_types(type_engine);
        all_types.insert(type_id);
        let all_types = all_types.into_iter().collect::<Vec<_>>();
        self.filter_by_type_inner(type_engine, all_types, decider)
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
        type_engine: &TypeEngine,
    ) -> TraitMap {
        // a curried version of the decider protocol to use in the helper functions
        let decider = |type_info: &TypeInfo, map_type_info: &TypeInfo| {
            type_info.is_subset_of(map_type_info, type_engine)
                || map_type_info.is_subset_of_for_item_import(type_info, type_engine)
        };
        let mut trait_map = self.filter_by_type_inner(type_engine, vec![type_id], decider);
        let all_types = type_engine
            .look_up_type_id(type_id)
            .extract_inner_types(type_engine)
            .into_iter()
            .collect::<Vec<_>>();
        // a curried version of the decider protocol to use in the helper functions
        let decider2 = |type_info: &TypeInfo, map_type_info: &TypeInfo| {
            type_info.is_subset_of(map_type_info, type_engine)
        };
        trait_map.extend(
            self.filter_by_type_inner(type_engine, all_types, decider2),
            type_engine,
        );
        trait_map
    }

    fn filter_by_type_inner(
        &self,
        type_engine: &TypeEngine,
        mut all_types: Vec<TypeId>,
        decider: impl Fn(&TypeInfo, &TypeInfo) -> bool,
    ) -> TraitMap {
        let mut trait_map = TraitMap::default();
        for TraitEntry {
            key:
                TraitKey {
                    name: map_trait_name,
                    type_id: map_type_id,
                },
            value: map_trait_methods,
        } in self.trait_impls.iter()
        {
            for type_id in all_types.iter_mut() {
                let type_info = type_engine.look_up_type_id(*type_id);
                if !type_info.can_change() && *type_id == *map_type_id {
                    trait_map.insert_inner(
                        map_trait_name.clone(),
                        *type_id,
                        map_trait_methods.clone(),
                        type_engine,
                    );
                } else if decider(&type_info, &type_engine.look_up_type_id(*map_type_id)) {
                    let type_mapping =
                        TypeMapping::from_superset_and_subset(type_engine, *map_type_id, *type_id);
                    let new_self_type = type_engine.insert_type(TypeInfo::SelfType);
                    type_id.replace_self_type(type_engine, new_self_type);
                    let trait_methods: TraitMethods = map_trait_methods
                        .clone()
                        .into_iter()
                        .map(|(name, decl_id)| {
                            let mut decl = de_look_up_decl_id(decl_id.clone());
                            decl.copy_types(&type_mapping, type_engine);
                            decl.replace_self_type(type_engine, new_self_type);
                            (name, de_insert(decl, decl_id.span()).with_parent(decl_id))
                        })
                        .collect();
                    trait_map.insert_inner(
                        map_trait_name.clone(),
                        *type_id,
                        trait_methods,
                        type_engine,
                    );
                }
            }
        }
        trait_map
    }

    /// Filters the contents of `self` to exclude elements that are superset
    /// types of the given `type_id`. This function is used when handling trait
    /// constraints and is coupled with `filter_by_type` and
    /// `filter_by_type_item_import`.
    pub(crate) fn filter_against_type(&mut self, type_engine: &TypeEngine, type_id: TypeId) {
        self.trait_impls.retain(|e| {
            !type_engine
                .look_up_type_id(type_id)
                .is_subset_of(&type_engine.look_up_type_id(e.key.type_id), type_engine)
        });
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
        type_engine: &TypeEngine,
        type_id: TypeId,
    ) -> Vec<DeclarationId> {
        let mut methods = vec![];
        // small performance gain in bad case
        if type_engine
            .look_up_type_id(type_id)
            .eq(&TypeInfo::ErrorRecovery, type_engine)
        {
            return methods;
        }
        for entry in self.trait_impls.iter() {
            if are_equal_minus_dynamic_types(type_engine, type_id, entry.key.type_id) {
                let mut trait_methods = entry
                    .value
                    .values()
                    .cloned()
                    .into_iter()
                    .collect::<Vec<_>>();
                methods.append(&mut trait_methods);
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
        type_engine: &TypeEngine,
        type_id: TypeId,
        trait_name: &CallPath,
    ) -> Vec<DeclarationId> {
        let mut methods = vec![];
        // small performance gain in bad case
        if type_engine
            .look_up_type_id(type_id)
            .eq(&TypeInfo::ErrorRecovery, type_engine)
        {
            return methods;
        }
        for e in self.trait_impls.iter() {
            let map_trait_name = CallPath {
                prefixes: e.key.name.prefixes.clone(),
                suffix: e.key.name.suffix.name.clone(),
                is_absolute: e.key.name.is_absolute,
            };
            if &map_trait_name == trait_name
                && are_equal_minus_dynamic_types(type_engine, type_id, e.key.type_id)
            {
                let mut trait_methods = e.value.values().cloned().into_iter().collect::<Vec<_>>();
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
        type_engine: &TypeEngine,
    ) -> CompileResult<()> {
        let warnings = vec![];
        let mut errors = vec![];

        let required_traits: BTreeSet<Ident> = constraints
            .iter()
            .cloned()
            .map(|constraint| constraint.trait_name.suffix)
            .collect();
        let mut found_traits: BTreeSet<Ident> = BTreeSet::new();

        for constraint in constraints.iter() {
            let TraitConstraint {
                trait_name: constraint_trait_name,
                type_arguments: constraint_type_arguments,
            } = constraint;
            let constraint_type_id = type_engine.insert_type(TypeInfo::Custom {
                name: constraint_trait_name.suffix.clone(),
                type_arguments: if constraint_type_arguments.is_empty() {
                    None
                } else {
                    Some(constraint_type_arguments.clone())
                },
            });
            for key in self.trait_impls.iter().map(|e| &e.key) {
                let suffix = &key.name.suffix;
                let map_trait_type_id = type_engine.insert_type(TypeInfo::Custom {
                    name: suffix.name.clone(),
                    type_arguments: if suffix.args.is_empty() {
                        None
                    } else {
                        Some(suffix.args.to_vec())
                    },
                });
                if are_equal_minus_dynamic_types(type_engine, type_id, key.type_id)
                    && are_equal_minus_dynamic_types(
                        type_engine,
                        constraint_type_id,
                        map_trait_type_id,
                    )
                {
                    found_traits.insert(constraint_trait_name.suffix.clone());
                }
            }
        }

        for trait_name in required_traits.difference(&found_traits) {
            // TODO: use a better span
            errors.push(CompileError::TraitConstraintNotSatisfied {
                ty: type_engine.help_out(type_id).to_string(),
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

fn are_equal_minus_dynamic_types(type_engine: &TypeEngine, left: TypeId, right: TypeId) -> bool {
    if left.index() == right.index() {
        return true;
    }
    match (
        type_engine.look_up_type_id(left),
        type_engine.look_up_type_id(right),
    ) {
        // these cases are false because, unless left and right have the same
        // TypeId, they may later resolve to be different types in the type
        // engine
        (TypeInfo::Unknown, TypeInfo::Unknown) => false,
        (TypeInfo::SelfType, TypeInfo::SelfType) => false,
        (TypeInfo::Numeric, TypeInfo::Numeric) => false,
        (TypeInfo::Contract, TypeInfo::Contract) => false,
        (TypeInfo::Storage { .. }, TypeInfo::Storage { .. }) => false,

        // these cases are able to be directly compared
        (TypeInfo::Boolean, TypeInfo::Boolean) => true,
        (TypeInfo::B256, TypeInfo::B256) => true,
        (TypeInfo::ErrorRecovery, TypeInfo::ErrorRecovery) => true,
        (TypeInfo::Str(l), TypeInfo::Str(r)) => l == r,
        (TypeInfo::UnsignedInteger(l), TypeInfo::UnsignedInteger(r)) => l == r,
        (TypeInfo::RawUntypedPtr, TypeInfo::RawUntypedPtr) => true,
        (TypeInfo::RawUntypedSlice, TypeInfo::RawUntypedSlice) => true,
        (TypeInfo::UnknownGeneric { .. }, TypeInfo::UnknownGeneric { .. }) => {
            // return true if left and right were unified previously
            type_engine.get_unified_types(left).contains(&right)
                || type_engine.get_unified_types(right).contains(&left)
        }

        // these cases may contain dynamic types
        (
            TypeInfo::Custom {
                name: l_name,
                type_arguments: l_type_args,
            },
            TypeInfo::Custom {
                name: r_name,
                type_arguments: r_type_args,
            },
        ) => {
            l_name == r_name
                && l_type_args
                    .unwrap_or_default()
                    .iter()
                    .zip(r_type_args.unwrap_or_default().iter())
                    .fold(true, |acc, (left, right)| {
                        acc && are_equal_minus_dynamic_types(
                            type_engine,
                            left.type_id,
                            right.type_id,
                        )
                    })
        }
        (
            TypeInfo::Enum {
                name: l_name,
                variant_types: l_variant_types,
                type_parameters: l_type_parameters,
            },
            TypeInfo::Enum {
                name: r_name,
                variant_types: r_variant_types,
                type_parameters: r_type_parameters,
            },
        ) => {
            l_name == r_name
                && l_variant_types.iter().zip(r_variant_types.iter()).fold(
                    true,
                    |acc, (left, right)| {
                        acc && left.name == right.name
                            && are_equal_minus_dynamic_types(
                                type_engine,
                                left.type_id,
                                right.type_id,
                            )
                    },
                )
                && l_type_parameters.iter().zip(r_type_parameters.iter()).fold(
                    true,
                    |acc, (left, right)| {
                        acc && left.name_ident == right.name_ident
                            && are_equal_minus_dynamic_types(
                                type_engine,
                                left.type_id,
                                right.type_id,
                            )
                    },
                )
        }
        (
            TypeInfo::Struct {
                name: l_name,
                fields: l_fields,
                type_parameters: l_type_parameters,
            },
            TypeInfo::Struct {
                name: r_name,
                fields: r_fields,
                type_parameters: r_type_parameters,
            },
        ) => {
            l_name == r_name
                && l_fields
                    .iter()
                    .zip(r_fields.iter())
                    .fold(true, |acc, (left, right)| {
                        acc && left.name == right.name
                            && are_equal_minus_dynamic_types(
                                type_engine,
                                left.type_id,
                                right.type_id,
                            )
                    })
                && l_type_parameters.iter().zip(r_type_parameters.iter()).fold(
                    true,
                    |acc, (left, right)| {
                        acc && left.name_ident == right.name_ident
                            && are_equal_minus_dynamic_types(
                                type_engine,
                                left.type_id,
                                right.type_id,
                            )
                    },
                )
        }
        (TypeInfo::Tuple(l), TypeInfo::Tuple(r)) => {
            if l.len() != r.len() {
                false
            } else {
                l.iter().zip(r.iter()).fold(true, |acc, (left, right)| {
                    acc && are_equal_minus_dynamic_types(type_engine, left.type_id, right.type_id)
                })
            }
        }
        (
            TypeInfo::ContractCaller {
                abi_name: l_abi_name,
                address: l_address,
            },
            TypeInfo::ContractCaller {
                abi_name: r_abi_name,
                address: r_address,
            },
        ) => {
            l_abi_name == r_abi_name
                && Option::zip(l_address, r_address)
                    .map(|(l_address, r_address)| {
                        are_equal_minus_dynamic_types(
                            type_engine,
                            l_address.return_type,
                            r_address.return_type,
                        )
                    })
                    .unwrap_or(true)
        }
        (TypeInfo::Array(l0, l1), TypeInfo::Array(r0, r1)) => {
            l1.len == r1.len && are_equal_minus_dynamic_types(type_engine, l0.type_id, r0.type_id)
        }
        _ => false,
    }
}
