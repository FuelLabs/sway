use std::fmt;

use crate::{
    language::{ty, CallPath},
    type_system::{look_up_type_id, CopyTypes, TypeId},
    TypeInfo, TypeMapping,
};

type TraitName = CallPath;
/// Map of function name to [TyFunctionDeclaration](ty::TyFunctionDeclaration)
type TraitMethods = im::HashMap<String, ty::TyFunctionDeclaration>;
/// Map of trait name and type to [TraitMethods].
type TraitImpls = im::HashMap<(TraitName, TypeId), TraitMethods>;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TraitMap {
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

    pub(crate) fn insert_for_type(&mut self, type_id: TypeId) {
        // println!("insert_for_type: {}", type_id);
        // println!("during: {}", self);
        // let trait_map = self.filter_by_type(type_id);
        // self.extend(trait_map);
        self.extend(self.filter_by_type_recursively(type_id));
        // for type_id in look_up_type_id(type_id).extract_inner_types().into_iter() {
        //     self.extend(self.filter_by_type(type_id));
        // }
    }

    pub(crate) fn extend(&mut self, other: TraitMap) {
        // println!("extend: {} traits", other.trait_impls.len());
        for (key, other_trait_methods) in other.trait_impls.into_iter() {
            match self.trait_impls.get_mut(&key) {
                Some(trait_methods) => {
                    trait_methods.extend(other_trait_methods.into_iter());
                }
                None => {
                    self.trait_impls.insert(key, other_trait_methods);
                }
            }
        }
    }

    // pub(crate) fn filter_by_type_recursively(&self, type_id: TypeId) -> TraitMap {
    //     // let mut trait_impls: TraitImpls = im::HashMap::new();
    //     // for ((map_trait_name, map_type_id), map_trait_impls) in self.trait_impls.iter() {
    //     //     if look_up_type_id(type_id).is_subset_of(&look_up_type_id(*map_type_id)) {
    //     //         trait_impls.insert((map_trait_name.clone(), type_id), map_trait_impls.clone());
    //     //     }
    //     // }
    //     let mut all_types = im::HashSet::new();
    //     all_types.insert(type_id);
    //     all_types.extend(look_up_type_id(type_id).extract_inner_types());
    //     let all_types = all_types.into_iter().collect::<Vec<_>>();

    //     let mut trait_map = TraitMap { trait_impls: Default::default() };
    //     for ((map_trait_name, map_type_id), map_trait_methods) in self.trait_impls.iter() {
    //         for type_id in all_types.iter() {
    //             if look_up_type_id(*type_id).is_subset_of(&look_up_type_id(*map_type_id)) {
    //                 // println!("\tfound subsets:\n\t\t{}\n\t\t{}", type_id, map_type_id);
    //                 let type_mapping = TypeMapping::from_superset_and_subset(*map_type_id, *type_id);
    //                 let mut trait_methods = map_trait_methods.values().cloned().into_iter().collect::<Vec<_>>();
    //                 trait_methods.iter_mut().for_each(|trait_method| {
    //                     // println!("before copy types: {} with {}", method, type_mapping);
    //                     trait_method.copy_types(&type_mapping);
    //                     trait_map.extend(self.filter_by_type_recursively(trait_method.return_type));
    //                     // println!("after copy types: {} with {}", method, type_mapping);
    //                 });
    //                 trait_map.insert(map_trait_name.clone(), *type_id, trait_methods);
    //             }
    //         }
    //     }
    //     println!(
    //         "filter_by_type: {} -------\n{}",
    //         type_id,
    //         trait_map
    //     );
    //     trait_map
    // }

    pub(crate) fn filter_by_type_recursively(&self, type_id: TypeId) -> TraitMap {
        fn helper(
            trait_map_self: &TraitMap,
            type_id: TypeId,
            seen: &mut im::HashSet<TypeId>,
        ) -> TraitMap {
            println!("helper for {}, {}", *type_id, type_id);
            seen.insert(type_id);
            let all_types = TraitMap::calculate_all_types(type_id);
            let mut trait_map = TraitMap {
                trait_impls: Default::default(),
            };

            for ((map_trait_name, map_type_id), map_trait_methods) in
                trait_map_self.trait_impls.iter()
            {
                for type_id in all_types.iter() {
                    if look_up_type_id(*type_id).is_subset_of(&look_up_type_id(*map_type_id)) {
                        let type_mapping =
                            TypeMapping::from_superset_and_subset(*map_type_id, *type_id);
                        let mut trait_methods = map_trait_methods
                            .values()
                            .cloned()
                            .into_iter()
                            .collect::<Vec<_>>();
                        trait_methods.iter_mut().for_each(|trait_method| {
                            let seen_it = seen.contains(&trait_method.return_type);
                            seen.insert(trait_method.return_type);
                            trait_method.copy_types(&type_mapping);
                            if !seen_it {
                                seen.insert(trait_method.return_type);
                                trait_map.extend(helper(
                                    trait_map_self,
                                    trait_method.return_type,
                                    seen,
                                ));
                            }
                        });
                        trait_map.insert(map_trait_name.clone(), *type_id, trait_methods);
                    }
                }
            }

            trait_map
        }

        let mut seen = im::HashSet::new();
        helper(self, type_id, &mut seen)
    }

    fn calculate_all_types(type_id: TypeId) -> Vec<TypeId> {
        let mut all_types = im::HashSet::new();
        all_types.insert(type_id);
        all_types.extend(look_up_type_id(type_id).extract_inner_types());
        all_types.into_iter().collect::<Vec<_>>()
    }

    pub(crate) fn get_methods_for_type(&self, type_id: TypeId) -> Vec<ty::TyFunctionDeclaration> {
        println!("get_methods_for_type: {}", type_id);
        println!("during: {}", self);
        let mut methods = vec![];
        // small performance gain in bad case
        if look_up_type_id(type_id) == TypeInfo::ErrorRecovery {
            return methods;
        }
        for ((_, map_type_id), map_trait_methods) in self.trait_impls.iter() {
            if look_up_type_id(type_id) == look_up_type_id(*map_type_id) {
                let mut trait_methods = map_trait_methods
                    .values()
                    .cloned()
                    .into_iter()
                    .collect::<Vec<_>>();
                methods.append(&mut trait_methods);
            }
        }
        // println!("found {} methods", methods.len());
        methods
    }
}
