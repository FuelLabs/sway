use std::fmt;

use crate::{
    insert_type,
    language::{ty, CallPath},
    type_system::{look_up_type_id, CopyTypes, TypeId},
    ReplaceSelfType, TypeInfo, TypeMapping,
};

type TraitName = CallPath;
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

impl fmt::Display for TraitMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TraitMap [\n\t{}\n]",
            self.trait_impls
                .iter()
                .map(|trait_impl| {
                    let ((trait_name, type_id), trait_methods) = trait_impl;
                    format!(
                        "impl {} for {} {{\n\t\t{}\n\t}}",
                        trait_name,
                        type_id,
                        trait_methods
                            .iter()
                            .map(|(_, method)| method.to_string())
                            .collect::<Vec<_>>()
                            .join("\n\t\t")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n\t")
        )
    }
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
        trait_name: TraitName,
        type_id: TypeId,
        methods: Vec<ty::TyFunctionDeclaration>,
    ) {
        let trait_methods: TraitMethods = methods
            .into_iter()
            .map(|method| (method.name.as_str().to_string(), method))
            .collect();
        let trait_impls: TraitImpls = vec![((trait_name, type_id), trait_methods)]
            .into_iter()
            .collect();
        let trait_map = TraitMap { trait_impls };
        self.extend(trait_map);
    }

    /// Given a [TypeId] `type_id`, retrieve entries in the [TraitMap] `self`
    /// for which `type_id` is a subset and re-insert them under `type_id`.
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
        for type_id in look_up_type_id(type_id).extract_inner_types().into_iter() {
            self.extend(self.filter_by_type(type_id));
        }
        // println!("{}", self);
    }

    pub(crate) fn extend(&mut self, other: TraitMap) {
        for (key, other_trait_methods) in other.trait_impls.into_iter() {
            self.trait_impls
                .entry(key)
                .or_insert(other_trait_methods.clone())
                .extend(other_trait_methods.into_iter());
        }
    }

    pub(crate) fn filter_by_type(&self, mut type_id: TypeId) -> TraitMap {
        let mut trait_map = TraitMap::default();
        for ((map_trait_name, map_type_id), map_trait_methods) in self.trait_impls.iter() {
            let type_info = look_up_type_id(type_id);
            if !type_info.can_change() && type_id == *map_type_id {
                let trait_methods = map_trait_methods
                    .values()
                    .cloned()
                    .into_iter()
                    .collect::<Vec<_>>();
                trait_map.insert(map_trait_name.clone(), type_id, trait_methods);
            } else if type_info.is_subset_of(&look_up_type_id(*map_type_id)) {
                let type_mapping = TypeMapping::from_superset_and_subset(*map_type_id, type_id);
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
                trait_map.insert(map_trait_name.clone(), type_id, trait_methods);
            }
        }
        trait_map
    }

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

        // these cases are able to be directly compared
        (TypeInfo::Boolean, TypeInfo::Boolean) => true,
        (TypeInfo::B256, TypeInfo::B256) => true,
        (TypeInfo::Contract, TypeInfo::Contract) => true,
        (TypeInfo::ErrorRecovery, TypeInfo::ErrorRecovery) => true,
        (TypeInfo::Str(l), TypeInfo::Str(r)) => l == r,
        (TypeInfo::UnsignedInteger(l), TypeInfo::UnsignedInteger(r)) => l == r,

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
        (TypeInfo::Storage { fields: l_fields }, TypeInfo::Storage { fields: r_fields }) => {
            l_fields
                .iter()
                .zip(r_fields.iter())
                .fold(true, |acc, (left, right)| {
                    acc && left.name == right.name
                        && are_equal_minus_dynamic_types(left.type_id, right.type_id)
                })
        }
        _ => false,
    }
}
