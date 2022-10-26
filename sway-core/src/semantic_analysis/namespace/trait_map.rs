use sway_error::error::CompileError;
use sway_types::{Ident, Span};

use crate::{
    error::*,
    insert_type,
    language::{ty, CallPath},
    type_system::{look_up_type_id, CopyTypes, TypeId},
    ReplaceSelfType, TypeArgument, TypeInfo, TypeMapping,
};

type TraitName = CallPath<(Ident, Vec<TypeArgument>)>;
/// Map of function name to [TyFunctionDeclaration](ty::TyFunctionDeclaration)
type TraitMethods = im::HashMap<String, ty::TyFunctionDeclaration>;
/// Map of trait name and type to [TraitMethods].
type TraitImpls = im::HashMap<(TraitName, TypeId), TraitMethods>;

/// Map holding trait implementations for types.
///
/// Note: "impl self" blocks are considered traits and are stored in the
/// [TraitMap].
#[derive(Clone, Debug, Default, PartialEq)]
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
    pub(crate) fn insert(
        &mut self,
        trait_name: CallPath,
        trait_type_args: Vec<TypeArgument>,
        type_id: TypeId,
        methods: Vec<ty::TyFunctionDeclaration>,
        impl_span: &Span,
    ) -> CompileResult<()> {
        let mut errors = vec![];

        // check to see if adding this trait will produce a conflicting definition
        let trait_type_id = insert_type(TypeInfo::Custom {
            name: trait_name.suffix.clone(),
            type_arguments: if trait_type_args.is_empty() {
                None
            } else {
                Some(trait_type_args.clone())
            },
        });
        for (map_trait_name, map_type_id) in self.trait_impls.keys() {
            let CallPath {
                suffix: (map_trait_name_suffix, map_trait_type_args),
                ..
            } = map_trait_name;
            let map_trait_type_id = insert_type(TypeInfo::Custom {
                name: map_trait_name_suffix.clone(),
                type_arguments: if map_trait_type_args.is_empty() {
                    None
                } else {
                    Some(map_trait_type_args.to_vec())
                },
            });
            let type_info = look_up_type_id(type_id);
            if look_up_type_id(trait_type_id).is_subset_of(&look_up_type_id(map_trait_type_id))
                && type_info.is_subset_of(&look_up_type_id(*map_type_id))
            {
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
                                .map(|type_arg| type_arg.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    }
                );
                errors.push(CompileError::ConflictingImplsForTraitAndType {
                    trait_name: trait_name_str,
                    type_implementing_for: type_id.to_string(),
                    second_impl_span: impl_span.clone(),
                });
            }
        }
        let trait_name: TraitName = CallPath {
            prefixes: trait_name.prefixes,
            suffix: (trait_name.suffix, trait_type_args),
            is_absolute: trait_name.is_absolute,
        };

        // even if there is a conflicting definition, add the trait anyway
        self.insert_inner(trait_name, type_id, methods);

        if errors.is_empty() {
            ok((), vec![], vec![])
        } else {
            err(vec![], errors)
        }
    }

    fn insert_inner(
        &mut self,
        trait_name: TraitName,
        type_id: TypeId,
        methods: Vec<ty::TyFunctionDeclaration>,
    ) {
        let trait_methods: TraitMethods = methods
            .into_iter()
            .map(|method| (method.name.as_str().to_string(), method))
            .collect();
        let trait_impls: TraitImpls =
            std::iter::once(((trait_name, type_id), trait_methods)).collect();
        let trait_map = TraitMap { trait_impls };
        self.extend(trait_map);
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
    pub(crate) fn insert_for_type(&mut self, type_id: TypeId) {
        self.extend(self.filter_by_type(type_id));
    }

    /// Given [TraitMap]s `self` and `other`, extend `self` with `other`,
    /// extending existing entries when possible.
    pub(crate) fn extend(&mut self, other: TraitMap) {
        for (key, other_trait_methods) in other.trait_impls.into_iter() {
            self.trait_impls
                .entry(key)
                .or_insert(other_trait_methods.clone())
                .extend(other_trait_methods.into_iter());
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
    pub(crate) fn filter_by_type(&self, type_id: TypeId) -> TraitMap {
        // a curried version of the decider protocol to use in the helper functions
        let decider =
            |type_info: &TypeInfo, map_type_info: &TypeInfo| type_info.is_subset_of(map_type_info);
        let mut all_types = look_up_type_id(type_id).extract_inner_types();
        all_types.insert(type_id);
        let all_types = all_types.into_iter().collect::<Vec<_>>();
        self.filter_by_type_inner(all_types, decider)
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
    pub(crate) fn filter_by_type_item_import(&self, type_id: TypeId) -> TraitMap {
        // a curried version of the decider protocol to use in the helper functions
        let decider = |type_info: &TypeInfo, map_type_info: &TypeInfo| {
            let hit =
                type_info.is_subset_of(map_type_info) || map_type_info.is_subset_of(type_info);
            if hit {
                println!("found hit: {} and {}", type_info, map_type_info);
            }
            hit
        };
        let mut trait_map = self.filter_by_type_inner(vec![type_id], decider);
        let all_types = look_up_type_id(type_id)
            .extract_inner_types()
            .into_iter()
            .collect::<Vec<_>>();
        let decider2 =
            |type_info: &TypeInfo, map_type_info: &TypeInfo| type_info.is_subset_of(map_type_info);
        trait_map.extend(self.filter_by_type_inner(all_types, decider2));
        trait_map
    }

    fn filter_by_type_inner<F>(&self, mut all_types: Vec<TypeId>, decider: F) -> TraitMap
    where
        F: Fn(&TypeInfo, &TypeInfo) -> bool,
    {
        let mut trait_map = TraitMap::default();
        for ((map_trait_name, map_type_id), map_trait_methods) in self.trait_impls.iter() {
            for type_id in all_types.iter_mut() {
                let type_info = look_up_type_id(*type_id);
                if !type_info.can_change() && *type_id == *map_type_id {
                    let trait_methods = map_trait_methods
                        .values()
                        .cloned()
                        .into_iter()
                        .collect::<Vec<_>>();
                    trait_map.insert_inner(map_trait_name.clone(), *type_id, trait_methods);
                } else if decider(&type_info, &look_up_type_id(*map_type_id)) {
                    let type_mapping =
                        TypeMapping::from_superset_and_subset(*map_type_id, *type_id);
                    let mut trait_methods = map_trait_methods
                        .values()
                        .cloned()
                        .into_iter()
                        .collect::<Vec<_>>();
                    trait_methods.iter_mut().for_each(|trait_method| {
                        trait_method.copy_types(&type_mapping);
                        let new_self_type = insert_type(TypeInfo::SelfType);
                        type_id.replace_self_type(new_self_type);
                        trait_method.replace_self_type(new_self_type);
                    });
                    trait_map.insert_inner(map_trait_name.clone(), *type_id, trait_methods);
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
    pub(crate) fn get_methods_for_type(&self, type_id: TypeId) -> Vec<ty::TyFunctionDeclaration> {
        let mut methods = vec![];
        // small performance gain in bad case
        if look_up_type_id(type_id) == TypeInfo::ErrorRecovery {
            return methods;
        }
        for ((_, map_type_id), map_trait_methods) in self.trait_impls.iter() {
            if are_equal_minus_dynamic_types(type_id, *map_type_id) {
                let mut trait_methods = map_trait_methods
                    .values()
                    .cloned()
                    .into_iter()
                    .collect::<Vec<_>>();
                methods.append(&mut trait_methods);
            }
        }
        methods
    }
}

fn are_equal_minus_dynamic_types(left: TypeId, right: TypeId) -> bool {
    if *left == *right {
        return true;
    }
    match (look_up_type_id(left), look_up_type_id(right)) {
        // these cases are false because, unless left and right have the same
        // TypeId, they may later resolve to be different types in the type
        // engine
        (TypeInfo::Unknown, TypeInfo::Unknown) => false,
        (TypeInfo::SelfType, TypeInfo::SelfType) => false,
        (TypeInfo::Numeric, TypeInfo::Numeric) => false,
        (TypeInfo::UnknownGeneric { .. }, TypeInfo::UnknownGeneric { .. }) => false,
        (TypeInfo::Contract, TypeInfo::Contract) => false,
        (TypeInfo::Storage { .. }, TypeInfo::Storage { .. }) => false,

        // these cases are able to be directly compared
        (TypeInfo::Boolean, TypeInfo::Boolean) => true,
        (TypeInfo::B256, TypeInfo::B256) => true,
        (TypeInfo::ErrorRecovery, TypeInfo::ErrorRecovery) => true,
        (TypeInfo::Str(l), TypeInfo::Str(r)) => l == r,
        (TypeInfo::UnsignedInteger(l), TypeInfo::UnsignedInteger(r)) => l == r,
        (TypeInfo::RawUntypedPtr, TypeInfo::RawUntypedPtr) => true,

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
                        acc && are_equal_minus_dynamic_types(left.type_id, right.type_id)
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
                            && are_equal_minus_dynamic_types(left.type_id, right.type_id)
                    },
                )
                && l_type_parameters.iter().zip(r_type_parameters.iter()).fold(
                    true,
                    |acc, (left, right)| {
                        acc && left.name_ident == right.name_ident
                            && are_equal_minus_dynamic_types(left.type_id, right.type_id)
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
                            && are_equal_minus_dynamic_types(left.type_id, right.type_id)
                    })
                && l_type_parameters.iter().zip(r_type_parameters.iter()).fold(
                    true,
                    |acc, (left, right)| {
                        acc && left.name_ident == right.name_ident
                            && are_equal_minus_dynamic_types(left.type_id, right.type_id)
                    },
                )
        }
        (TypeInfo::Tuple(l), TypeInfo::Tuple(r)) => {
            l.iter().zip(r.iter()).fold(true, |acc, (left, right)| {
                acc && are_equal_minus_dynamic_types(left.type_id, right.type_id)
            })
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
                        are_equal_minus_dynamic_types(l_address.return_type, r_address.return_type)
                    })
                    .unwrap_or(true)
        }
        (TypeInfo::Array(l0, l1, _), TypeInfo::Array(r0, r1, _)) => {
            l1 == r1 && are_equal_minus_dynamic_types(l0, r0)
        }
        _ => false,
    }
}
