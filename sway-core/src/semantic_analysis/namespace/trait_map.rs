use crate::{CallPath, TypeInfo, TypedFunctionDeclaration};

use std::collections::HashMap;

type TraitName = CallPath;

// This cannot be a HashMap because of how TypeInfo's are handled.
//
// In Rust, in general, a custom type should uphold the invariant
// that PartialEq and Hash produce consistent results. i.e. for
// two objects, their hash value is equal if and only if they are
// equal under the PartialEq trait.
//
// For TypeInfo, this means that if you have:
//
// ```ignore
// 1: u64
// 2: u64
// 3: Ref(1)
// 4: Ref(2)
// ```
//
// 1, 2, 3, 4 are equal under PartialEq and their hashes are the same
// value.
//
// However, we need this structure to be able to maintain the
// difference between 3 and 4, as in practice, 1 and 2 might not yet
// be resolved.
type TraitMapInner = im::Vector<((TraitName, TypeInfo), TraitMethods)>;
type TraitMethods = im::HashMap<String, TypedFunctionDeclaration>;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TraitMap {
    trait_map: TraitMapInner,
}

impl TraitMap {
    pub(crate) fn insert(
        &mut self,
        trait_name: CallPath,
        type_implementing_for: TypeInfo,
        methods: Vec<TypedFunctionDeclaration>,
    ) {
        let mut methods_map = im::HashMap::new();
        for method in methods.into_iter() {
            let method_name = method.name.as_str().to_string();
            methods_map.insert(method_name, method);
        }
        self.trait_map
            .push_back(((trait_name, type_implementing_for), methods_map));
    }

    pub(crate) fn extend(&mut self, other: TraitMap) {
        for ((trait_name, type_implementing_for), methods) in other.trait_map.into_iter() {
            self.insert(
                trait_name,
                type_implementing_for,
                methods.values().cloned().collect(),
            );
        }
    }

    pub(crate) fn get_call_path_and_type_info(
        &self,
        r#type: TypeInfo,
    ) -> Vec<((CallPath, TypeInfo), Vec<TypedFunctionDeclaration>)> {
        let mut ret = vec![];
        for ((call_path, type_info), methods) in self.trait_map.iter() {
            if type_info.clone() == r#type {
                ret.push((
                    (call_path.clone(), type_info.clone()),
                    methods.values().cloned().collect(),
                ));
            }
        }
        ret
    }

    pub(crate) fn get_methods_for_type(&self, r#type: TypeInfo) -> Vec<TypedFunctionDeclaration> {
        let mut methods = vec![];
        // small performance gain in bad case
        if r#type == TypeInfo::ErrorRecovery {
            return methods;
        }
        for ((_, type_info), l_methods) in self.trait_map.iter() {
            if *type_info == r#type {
                methods.append(&mut l_methods.values().cloned().collect());
            }
        }
        methods
    }

    pub(crate) fn get_methods_for_type_by_trait(
        &self,
        r#type: TypeInfo,
    ) -> HashMap<TraitName, Vec<TypedFunctionDeclaration>> {
        let mut methods: HashMap<TraitName, Vec<TypedFunctionDeclaration>> = HashMap::new();
        // small performance gain in bad case
        if r#type == TypeInfo::ErrorRecovery {
            return methods;
        }
        for ((trait_name, type_info), trait_methods) in self.trait_map.iter() {
            if *type_info == r#type {
                methods.insert(
                    (*trait_name).clone(),
                    trait_methods.values().cloned().collect(),
                );
            }
        }
        methods
    }
}
